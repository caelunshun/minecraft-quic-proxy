use anyhow::{anyhow, Context as _};
use jni::{
    objects::{JByteArray, JClass, JString},
    sys::{jint, jlong},
    JNIEnv,
};
use minecraft_quic_proxy::{
    client::ClientHandle,
    quinn::{ClientConfig, Endpoint},
};
use std::{convert::identity, panic, panic::AssertUnwindSafe, sync::Arc};
use tokio::{runtime, runtime::Runtime};

unsafe fn deref_from_long<'a, T>(long: jlong) -> &'a T {
    unsafe { &*(long as *const T) }
}

struct Context {
    runtime: Runtime,
    endpoint: Endpoint,
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicContext_init(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    wrap_with_error_handling(&mut env, |_env| {
        tracing_subscriber::fmt()
            .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
            .with_ansi(false)
            .try_init()
            .ok();
        std::env::set_var("RUST_BACKTRACE", "1");

        let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
        let _guard = runtime.enter();

        #[cfg(feature = "ignore-server-certificates")]
        let mut client_config = {
            let crypto = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
                .with_no_client_auth();
            ClientConfig::new(Arc::new(crypto))
        };
        #[cfg(not(feature = "ignore-server-certificates"))]
        let mut client_config = ClientConfig::with_native_roots();

        client_config.transport_config(Arc::new(minecraft_quic_proxy::transport_config()));

        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        let context = Box::new(Context { runtime, endpoint });
        Ok(Box::into_raw(context) as jlong)
    })
}

#[cfg(feature = "ignore-server-certificates")]
struct SkipServerVerification;

#[cfg(feature = "ignore-server-certificates")]
impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicContext_createClient(
    mut env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
    gateway_host: JString,
    gateway_port: jint,
    destination_address: JString,
    authentication_key: JString,
) -> jlong {
    wrap_with_error_handling(&mut env, |env| {
        let context = deref_from_long::<Context>(context_ptr);
        let destination_address = env
            .get_string(&destination_address)?
            .to_string_lossy()
            .into_owned();
        let authentication_key = env
            .get_string(&authentication_key)?
            .to_string_lossy()
            .into_owned();
        let gateway_host = env
            .get_string(&gateway_host)?
            .to_string_lossy()
            .into_owned();

        let destination_address = destination_address.parse()?;
        let client = context.runtime.block_on(async move {
            ClientHandle::open(
                &context.endpoint,
                &gateway_host,
                gateway_port as u16,
                destination_address,
                &authentication_key,
            )
            .await
            .context("failed to connect to gateway")
        })?;

        Ok(Box::into_raw(Box::new(client)) as jlong)
    })
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicContext_drop(
    mut env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) {
    wrap_with_error_handling(&mut env, |_| {
        drop(Box::from_raw(context_ptr as *mut Context));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicClient_getPort(
    _env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
) -> jint {
    let client: &ClientHandle = deref_from_long(client_ptr);
    client.bound_port() as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicClient_enableEncryption(
    mut env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    jkey: JByteArray,
) {
    wrap_with_error_handling(&mut env, |env| {
        let mut key = [0i8; 16];
        env.get_byte_array_region(jkey, 0, &mut key).unwrap();
        let client: &mut ClientHandle = &mut *(client_ptr as *mut ClientHandle);
        client.set_encryption_key(key.map(|x| x as u8));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicClient_drop(
    mut env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
) {
    wrap_with_error_handling(&mut env, |_| {
        drop(Box::from_raw(client_ptr as *mut ClientHandle));
        Ok(())
    })
}

fn wrap_with_error_handling<R: Default>(
    env: &mut JNIEnv,
    callback: impl FnOnce(&mut JNIEnv) -> anyhow::Result<R>,
) -> R {
    let result = panic::catch_unwind(AssertUnwindSafe(|| callback(env)));

    let result = result
        .map_err(|_| anyhow!("Rust panic occurred"))
        .and_then(identity);

    match result {
        Ok(r) => r,
        Err(e) => {
            env.throw_new("java/lang/RuntimeException", e.to_string())
                .unwrap();
            R::default()
        }
    }
}

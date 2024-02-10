use jni::{
    objects::{JByteArray, JClass, JString},
    sys::{jint, jlong},
    JNIEnv,
};
use minecraft_quic_proxy::{
    client::ClientHandle,
    quinn::{ClientConfig, Endpoint},
};
use std::sync::Arc;
use tokio::{runtime, runtime::Runtime};
use tracing_subscriber::filter::LevelFilter;

const GATEWAY_HOST: &str = "127.0.0.1";
const GATEWAY_PORT: u16 = 6666;

unsafe fn deref_from_long<'a, T>(long: jlong) -> &'a T {
    unsafe { &*(long as *const T) }
}

struct Context {
    runtime: Runtime,
    endpoint: Endpoint,
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicContext_init(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .try_init()
        .ok();
    std::env::set_var("RUST_BACKTRACE", "1");

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    let _guard = runtime.enter();

    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();
    let client_config = ClientConfig::new(Arc::new(crypto));

    let mut endpoint =
        Endpoint::client("0.0.0.0:0".parse().unwrap()).expect("failed to bind endpoint");
    endpoint.set_default_client_config(client_config);

    let context = Box::new(Context { runtime, endpoint });
    Box::into_raw(context) as jlong
}

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

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
    destination_address: JString,
    authentication_key: JString,
) -> jlong {
    let context = deref_from_long::<Context>(context_ptr);
    let destination_address = env
        .get_string(&destination_address)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let authentication_key = env
        .get_string(&authentication_key)
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let client = context.runtime.block_on(async move {
        ClientHandle::open(
            &context.endpoint,
            GATEWAY_HOST,
            GATEWAY_PORT,
            destination_address.parse().unwrap(),
            &authentication_key,
        )
        .await
        .expect("failed to connect to gateway")
    });

    Box::into_raw(Box::new(client)) as jlong
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicContext_drop(
    _env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) {
    drop(Box::from_raw(context_ptr as *mut Context));
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
    env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    jkey: JByteArray,
) {
    let mut key = [0i8; 16];
    env.get_byte_array_region(jkey, 0, &mut key).unwrap();
    let client: &mut ClientHandle = &mut *(client_ptr as *mut ClientHandle);
    client.set_encryption_key(key.map(|x| x as u8));
}

#[no_mangle]
pub unsafe extern "system" fn Java_me_caelunshun_quicproxy_jni_RustQuicClient_drop(
    _env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
) {
    drop(Box::from_raw(client_ptr as *mut ClientHandle))
}

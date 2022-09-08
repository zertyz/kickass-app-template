//! Please, see [super]

use crate::frontend::{
    telegram::TelegramUI,
    web::WebServer,
};
use std::time::{SystemTime,Duration};
use futures::future::BoxFuture;
use log::debug;
use tokio::sync::RwLock;

/// Timeout to wait for `Option` data to be filled in -- when retrieving it
const TIMEOUT: Duration = Duration::from_secs(3);
/// Time to wait on between checks for an `Option` data to be filled in -- when retrieving it
const POLL_INTERVAL: Duration = Duration::from_micros(1000);


/// Contains data filled at runtime -- not present in the config file
pub struct Runtime {

    // environment
    //////////////

    /// this process executable's absolute path, used o determine the
    /// date the executable was generated, which is important for optimization
    /// decisions regarding the need for datasets & amalgamations to be regenerated
    pub executable_path: String,

    // internal task communication
    //////////////////////////////

    /// The Telegram controller -- can be used to send push messages & request the telegram service to shutdown
    /// -- see [TelegramUI]
    telegram_ui: Option<TelegramUI>,

    /// The Rocket controller -- can be used to inquiring the running state and to request the service to shutdown
    /// -- See [WebServer]
    web_server: Option<WebServer>,

}

/// Macro to create getters & setters for `Option` fields -- with timeouts and dead-lock prevention
macro_rules! impl_runtime {
    ($field_name_str:    literal,
     $field_name_ident:  ident,
     $field_type:        ty,
     $set_function_name: ident,
     $get_function_name: ident) => {

        impl Runtime {

            /// RW-Locks `runtime`, then registers the [Runtime::$field_name_ident] -- so it may be retrieved (possibly in another thread) with [$get_function_name()]\
            ///
            /// Example:
            /// ```no_compile
            ///     Runtime::$set_function_name(&runtime, $field_name_ident).await;
            pub async fn $set_function_name(runtime: &RwLock<Self>, $field_name_ident: $field_type) {
                runtime.write().await.$field_name_ident.replace($field_name_ident);
            }

            /// Gets (or waits for up to a reasonable, hard-coded timeout) the [Runtime::$field_name_ident] -- as set (possibly in another thread or task)
            /// by [$set_function_name()] -- then pass it to `callback()` to do something useful with it while `runtime` is read-locked\
            ///
            /// Example:
            /// ```no_compile
            ///     Runtime::$get_function_name(&runtime, |$field_name, _runtime| Box::pin(async move {
            ///         $field_name.broadcast_message(&contents_for_$field_name, true).await
            ///     })).await?;
            pub async fn $get_function_name<ReturnType>
                                           (runtime:  &RwLock<Self>,
                                            callback: impl for<'r> FnOnce(&'r $field_type, &'r Runtime) -> BoxFuture<'r, ReturnType> + Send)
                                           -> ReturnType {
                let mut start: Option<SystemTime> = None;
                loop {
                    if let Ok(runtime) = &runtime.try_read() {
                        if let Some($field_name_ident) = &runtime.$field_name_ident {
                            if let Some(start) = start {
                                debug!("Runtime: `{}` became available after a {:?} wait", $field_name_str, start.elapsed().unwrap());
                            }
                            break callback(&$field_name_ident, &runtime).await
                        }
                    }
                    if let Some(_start) = start {
                        if _start.elapsed().unwrap() > TIMEOUT {
                            panic!("Could not retrieve `{}` instance: {}",
                                   $field_name_str,
                                   if let Ok(_runtime) = &runtime.try_read() {
                                       format!("`Runtime` seems to be locked elsewhere for the past {:?}", TIMEOUT)
                                   } else {
                                       format!("it was not registered in `Runtime` even after {:?}", TIMEOUT)
                                });
                        }
                    } else {
                        start = Some(SystemTime::now());
                        debug!("Runtime: `{}` is not (yet?) available. Waiting for up to {:?} for main.rs to finish instantiating it and placing it here with `register_{}()`",
                               $field_name_str, TIMEOUT, $field_name_str);
                    }
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            }

        }
    }
}

impl Runtime {

    pub fn new(executable_path: String) -> Self {
        Self {
            executable_path,
            telegram_ui: None,
            web_server:  None,
        }
    }
}

// implements getters and setters for all `Option` fields that are to be set/get asynchronously
///////////////////////////////////////////////////////////////////////////////////////////////
impl_runtime!("telegram_ui", telegram_ui, TelegramUI, register_telegram_ui, do_for_telegram_ui);
impl_runtime!("web_server",  web_server,  WebServer,  register_web_server,  do_for_web_server);

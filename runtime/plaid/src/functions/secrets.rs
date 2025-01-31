use super::{get_memory, safely_get_string, safely_write_data_back};
use crate::executor::Env;

use wasmer::{AsStoreRef, FunctionEnvMut, WasmPtr};

pub fn get_secrets(
    env: FunctionEnvMut<Env>,
    name_buf: WasmPtr<u8>,
    name_len: u32,
    data_buffer: WasmPtr<u8>,
    buffer_size: u32,
) -> i32 {
    let Some(secrets) = &env.data().module.secrets else {
        // This module has no secrets: we just return 0
        return 0;
    };
    let store = env.as_store_ref();
    let memory_view = match get_memory(&env, &store) {
        Ok(memory_view) => memory_view,
        Err(e) => {
            error!(
                "{}: Memory error in fetch_from_module: {:?}",
                env.data().module.name,
                e
            );
            return e as i32;
        }
    };

    let name = match safely_get_string(&memory_view, name_buf, name_len) {
        Ok(x) => x,
        Err(e) => {
            error!("{}: Error in get_secrets: {:?}", env.data().module.name, e);
            return e as i32;
        }
    };

    // Check if this field is present at all
    if let Some(data) = secrets.get(&name) {
        match safely_write_data_back(&memory_view, &data, data_buffer, buffer_size) {
            Ok(x) => x,
            Err(e) => {
                error!("{}: Error in get_secrets: {:?}", env.data().module.name, e);
                e as i32
            }
        }
    } else {
        // If there is no field with that name, we return 0 similar to
        // fetching the from_module
        0
    }
}

// Python値読み取り
use rustpython_vm::builtins::PyDictRef;
use rustpython_vm::VirtualMachine;

// 文字列取得
pub fn read_option_string(vm: &VirtualMachine, dict: &PyDictRef, key: &str, label: &str, warn_missing: bool) -> Option<String> {
    match dict.get_item_opt(key, vm) {
        Ok(Some(value)) => match value.try_into_value(vm) {
            Ok(result) => Some(result),
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: {} {} {:?}", label, key, err);
                None
            }
        },
        Ok(None) => {
            if warn_missing {
                println!("Warning: {} missing {}", label, key);
            }
            None
        }
        Err(err) => {
            vm.print_exception(err.clone());
            println!("Warning: {} {} {:?}", label, key, err);
            None
        }
    }
}

// f32取得
pub fn read_option_f32(vm: &VirtualMachine, dict: &PyDictRef, key: &str, label: &str, warn_missing: bool) -> Option<f32> {
    match dict.get_item_opt(key, vm) {
        Ok(Some(value)) => match value.try_into_value(vm) {
            Ok(result) => Some(result),
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: {} {} {:?}", label, key, err);
                None
            }
        },
        Ok(None) => {
            if warn_missing {
                println!("Warning: {} missing {}", label, key);
            }
            None
        }
        Err(err) => {
            vm.print_exception(err.clone());
            println!("Warning: {} {} {:?}", label, key, err);
            None
        }
    }
}

// i32取得
pub fn read_option_i32(vm: &VirtualMachine, dict: &PyDictRef, key: &str, label: &str, warn_missing: bool) -> Option<i32> {
    match dict.get_item_opt(key, vm) {
        Ok(Some(value)) => match value.try_into_value(vm) {
            Ok(result) => Some(result),
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: {} {} {:?}", label, key, err);
                None
            }
        },
        Ok(None) => {
            if warn_missing {
                println!("Warning: {} missing {}", label, key);
            }
            None
        }
        Err(err) => {
            vm.print_exception(err.clone());
            println!("Warning: {} {} {:?}", label, key, err);
            None
        }
    }
}

// 文字列配列取得
pub fn read_option_vec_string(vm: &VirtualMachine, dict: &PyDictRef, key: &str, label: &str, warn_missing: bool) -> Option<Vec<String>> {
    match dict.get_item_opt(key, vm) {
        Ok(Some(value)) => match value.try_into_value(vm) {
            Ok(result) => Some(result),
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: {} {} {:?}", label, key, err);
                None
            }
        },
        Ok(None) => {
            if warn_missing {
                println!("Warning: {} missing {}", label, key);
            }
            None
        }
        Err(err) => {
            vm.print_exception(err.clone());
            println!("Warning: {} {} {:?}", label, key, err);
            None
        }
    }
}

// f32配列取得
pub fn read_option_vec_f32(vm: &VirtualMachine, dict: &PyDictRef, key: &str, label: &str, warn_missing: bool) -> Option<Vec<f32>> {
    match dict.get_item_opt(key, vm) {
        Ok(Some(value)) => match value.try_into_value::<Option<Vec<f32>>>(vm) {
            Ok(result) => result,
            Err(err) => {
                vm.print_exception(err.clone());
                println!("Warning: {} {} {:?}", label, key, err);
                None
            }
        },
        Ok(None) => {
            if warn_missing {
                println!("Warning: {} missing {}", label, key);
            }
            None
        }
        Err(err) => {
            vm.print_exception(err.clone());
            println!("Warning: {} {} {:?}", label, key, err);
            None
        }
    }
}

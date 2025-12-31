# zsh-system

`zsh-system` is a Rust library designed to simplify the development of Zsh (Z Shell) modules using Rust. It provides safe and idiomatic Rust wrappers around Zsh's C API, allowing developers to create powerful and performant Zsh extensions with the safety and expressiveness of Rust.

## Features

- **Safe Zsh C API Bindings**: Automatically generated FFI bindings to the Zsh C API via `bindgen`.
- **Parameter Management**: Read, write, and unset Zsh shell parameters (strings, integers, arrays).
- **Memory Management**: Utilities (`ZBox`, `ZString`) for safe interaction with Zsh's memory allocators (`zalloc`, `zsfree`, `ztrdup`).
- **Module Lifecycle Integration**: A declarative macro (`export_module!`) to define Zsh module entry points (`setup_`, `boot_`, `cleanup_`, `finish_`) and manage module instances.
- **Feature Registration**: Define and register custom Zsh features:
    - **Builtin Commands**: Implement new shell commands directly in Rust.
    - **Conditional Expressions (`conddef`)**: Extend Zsh's `[[ ... ]]` conditional logic.
    - **Mathematical Functions (`mathfunc`)**: Add custom functions for arithmetic evaluation (`(( ... ))`).
    - **Parameter Definitions (`paramdef`)**: Create new shell variables with custom behavior.
- **Hook System Interaction**: Add, remove, and run Zsh hooks to extend shell behavior at specific events.
- **Script Evaluation**: Execute Zsh script strings directly from Rust code.

## Getting Started

### Prerequisites

- Rust toolchain (latest stable version recommended).
- Zsh development headers. On Debian/Ubuntu, install `zsh-dev`:
  ```bash
  sudo apt install zsh-dev
  ```
- Clang (required by `bindgen` for C header parsing).

### Building

To build the `zsh-system` library:

```bash
cargo build
```

This will produce a dynamic library (e.g., `libzsh_system.so` on Linux) that can be loaded by Zsh as a module.

### Example Zsh Module

Here's a minimal example of how to create a Zsh module in Rust:

```rust
use zsh_system::{export_module, ZshModule, ZshResult, Features};
use zsh_system::module::builtin::BuiltinHandler;

struct MyModule;

impl Default for MyModule {
    fn default() -> Self {
        MyModule
    }
}

impl ZshModule for MyModule {
    fn setup(&mut self) -> ZshResult {
        println!("MyModule: setup called!");
        Ok(())
    }

    fn boot(&mut self) -> ZshResult {
        println!("MyModule: boot called!");
        Ok(())
    }

    fn cleanup(&mut self) -> ZshResult {
        println!("MyModule: cleanup called!");
        Ok(())
    }

    fn finish(&mut self) -> ZshResult {
        println!("MyModule: finish called!");
        Ok(())
    }

    fn features(&self) -> Features {
        Features::new()
            .add_builtin("mybuiltin", my_builtin_handler)
    }
}

// Handler for the "mybuiltin" command
fn my_builtin_handler(name: &str, args: &[&str]) -> i32 {
    println!("Hello from mybuiltin! Name: {}, Args: {:?}", name, args);
    0 // Return 0 for success
}

// Export the module using the macro
export_module!(MyModule);
```

### Loading the Module in Zsh

1.  Compile your Rust module (e.g., `my_zsh_module`):
    ```bash
    # In your module's Cargo.toml, ensure it's a cdylib
    # [lib]
    # crate-type = ["cdylib"]
    cargo build --release
    ```
2.  In your Zsh configuration (`.zshrc` or similar), add:
    ```zsh
    # Assuming the compiled module is in target/release/
    module_path=(~/path/to/your/module/target/release $module_path)
    zmodload my_zsh_module
    ```
3.  Restart Zsh or source your `.zshrc`. You should now be able to use your custom builtins and features.

## Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License.
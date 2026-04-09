# nox-spirv
A compact SPIR-V reflection library written in pure-Rust with zero dependencies, minimal allocations and fast compile-times.

## Usage

An example of doing some reflection on SPIR-V words.
``` rust
use nox_spirv::op;
use nox_spirv::Module;
use nox_spirv::reflect::{Reflector, ResourceType};

let spirv: &[u32] = ...;
let module = Module::new(spirv);
let mut reflector = Reflector::new(module).unwrap();
for ubo in reflector.resources_for_type(ResourceType::UniformBuffer).unwrap() {
    match ubo {
        Ok(ubo) => {
            let mut set = None;
            let mut binding = None;
            for dec in reflector.decorations(ubo.variable_id) {
                if let op::Decoration::DescriptorSet { descriptor_set } = dec.decoration {
                    set = Some(descriptor_set);
                } else if let op::Decoration::Binding { binding_point } = dec.decoration {
                    binding = Some(binding_point);
                }
            }
            println!("Uniform buffer (set {}, binding {}): {}",
                set.unwrap(), binding.unwrap(), ubo.name.unwrap_or_default(),
            );
        },
        Err(err) => eprintln!("parse error: {err}")
    }
}
for pc in reflector.resources_for_type(ResourceType::PushConstant).unwrap() {
    match pc {
        Ok(pc) => {
            let size = reflector
                .type_description(pc.base_type_id)
                .unwrap().size_hint.declared();
            println!("Push constant (size {size}): {}", pc.name.unwrap_or_default());
        },
        Err(err) => eprintln!("parse error: {err}"),
    }
}
```

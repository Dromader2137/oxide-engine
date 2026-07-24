use crate::{
    assets::{Asset, AssetDescription},
    ecs::resources::Resources,
    error::OptionNoneErr,
    graphics::{
        ActiveGraphicsBackend,
        shader_module::{
            ShaderModule, ShaderModuleBuilder, ShaderType,
            reinterpret_shader_code,
        },
    },
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CookedShader {
    shader_type: ShaderType,
    code: Vec<u32>,
}

#[derive(Debug)]
pub struct Shader {
    pub shader_type: ShaderType,
    pub module: ShaderModule,
}

impl Asset for Shader {
    type Cooked = CookedShader;

    const ASSET_TYPE: &'static str = "shader";
    const EXTENSIONS: &'static [&'static str] = &["vert", "frag"];

    fn cook(data: &[u8], asset_description: &AssetDescription) -> Self::Cooked {
        let extension = asset_description
            .path
            .extension()
            .expect("shader has to have an extension")
            .to_str()
            .expect("shader has to have a valid utf-8 extension");
        // Compile GLSL -> SPV
        let text = String::from_utf8_lossy(data);
        let compiler = shaderc::Compiler::new().unwrap();
        let spv = compiler
            .compile_into_spirv(
                &text,
                match extension {
                    "vert" => shaderc::ShaderKind::DefaultVertex,
                    "frag" => shaderc::ShaderKind::DefaultFragment,
                    _ => panic!("invalid shader extension"),
                },
                "ast",
                "main",
                None,
            )
            .unwrap();
        let shader_type = match extension {
            "vert" => ShaderType::Vertex,
            "frag" => ShaderType::Fragment,
            _ => panic!("invalid shader extension"),
        };
        let code = reinterpret_shader_code(spv.as_binary_u8()).unwrap();
        return CookedShader { shader_type, code };
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self {
        let mut backend = resources
            .get_mut::<ActiveGraphicsBackend>()
            .none_err()
            .unwrap();
        let shader_module = ShaderModuleBuilder::default()
            .shader_type(cooked.shader_type)
            .code(&cooked.code)
            .build(&mut backend)
            .unwrap();
        Shader {
            shader_type: cooked.shader_type,
            module: shader_module,
        }
    }
}

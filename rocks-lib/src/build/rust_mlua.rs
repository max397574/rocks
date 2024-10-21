use eyre::{eyre, Result};
use indicatif::MultiProgress;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::progress::with_spinner;
use crate::rockspec::utils::lua_lib_extension;
use crate::{
    config::{Config, LuaVersion},
    lua_installation::LuaInstallation,
    rockspec::{Build, RustMluaBuildSpec},
    tree::RockLayout,
};

impl Build for RustMluaBuildSpec {
    async fn run(
        self,
        progress: &MultiProgress,
        output_paths: &RockLayout,
        _no_install: bool,
        _lua: &LuaInstallation,
        config: &Config,
        build_dir: &Path,
    ) -> Result<()> {
        let lua_version = LuaVersion::from(config)?;
        let lua_feature = match lua_version {
            LuaVersion::Lua51 => "lua51",
            LuaVersion::Lua52 => "lua52",
            LuaVersion::Lua53 => "lua53",
            LuaVersion::Lua54 => "lua54",
            LuaVersion::LuaJIT => "luajit",
            LuaVersion::LuaJIT52 => "luajit",
        };
        let features = self
            .features
            .into_iter()
            .chain(std::iter::once(lua_feature.into()))
            .join(",");
        let target_dir_arg = format!("--target-dir={}", self.target_path.display());
        let mut build_args = vec!["build", "--release", &target_dir_arg];
        if !self.default_features {
            build_args.push("--no-default-features");
        }
        build_args.push("--features");
        build_args.push(&features);
        match Command::new("cargo")
            .current_dir(build_dir)
            .args(build_args)
            .output()
        {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                return Err(eyre!(
                    "`cargo build` failed.
status: {}
stdout: {}
stderr: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ))
            }
            Err(err) => return Err(eyre!("Failed to run `cargo build`: {err}")),
        }
        fs::create_dir_all(&output_paths.lib)?;
        if let Err(err) =
            install_rust_libs(self.modules, &self.target_path, build_dir, output_paths)
        {
            cleanup(progress, output_paths).await?;
            return Err(err);
        }
        fs::create_dir_all(&output_paths.src)?;
        if let Err(err) = install_lua_libs(self.include, build_dir, output_paths) {
            cleanup(progress, output_paths).await?;
            return Err(err);
        }
        Ok(())
    }
}

fn install_rust_libs(
    modules: HashMap<String, PathBuf>,
    target_path: &Path,
    build_dir: &Path,
    output_paths: &RockLayout,
) -> Result<()> {
    for (module, rust_lib) in modules {
        let src = build_dir.join(target_path).join("release").join(rust_lib);
        let mut dst: PathBuf = output_paths.lib.join(module);
        dst.set_extension(lua_lib_extension());
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn install_lua_libs(
    include: HashMap<PathBuf, PathBuf>,
    build_dir: &Path,
    output_paths: &RockLayout,
) -> Result<()> {
    for (from, to) in include {
        let src = build_dir.join(from);
        let dst = output_paths.src.join(to);
        fs::copy(src, dst)?;
    }
    Ok(())
}

async fn cleanup(progress: &MultiProgress, output_paths: &RockLayout) -> Result<()> {
    let root_dir = &output_paths.rock_path;
    with_spinner(
        progress,
        format!("🗑️ Cleaning up {}", root_dir.display()),
        || async {
            match std::fs::remove_dir_all(root_dir) {
                Ok(_) => (),
                Err(err) => progress.println(format!(
                    "Error cleaning up {}: {}",
                    root_dir.display(),
                    err
                ))?,
            };
            Ok(())
        },
    )
    .await?;
    Ok(())
}

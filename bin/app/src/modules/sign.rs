use std::fs::{create_dir_all, File};

use git2::Repository;
use hemtt_bin_error::Error;
use hemtt_pbo::ReadablePbo;
use hemtt_signing::BIPrivateKey;

use crate::addons::Location;

use super::Module;

pub struct Sign;
impl Sign {
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Sign {
    fn name(&self) -> &'static str {
        "Sign"
    }

    fn check(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        if ctx.config().version().git_hash().is_some() {
            let _ = Repository::open(".")?;
        }
        Ok(())
    }

    fn pre_release(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        let authority = get_authority(ctx, None)?;
        let addons_key = BIPrivateKey::generate(1024, &authority)?;
        create_dir_all(ctx.hemtt_folder().join("keys"))?;
        addons_key.write(&mut File::create(
            ctx.hemtt_folder()
                .join("keys")
                .join(format!("{authority}.bikey")),
        )?)?;
        ctx.addons().to_vec().iter().try_for_each(|addon| {
            let pbo_name = addon.pbo_name(ctx.config().prefix());
            let (mut pbo, sig_location, key) = match addon.location() {
                Location::Addons => {
                    let target_pbo = {
                        let mut path = ctx.hemtt_folder().join("addons").join(pbo_name);
                        path.set_extension("pbo");
                        path
                    };
                    (
                        ReadablePbo::from(File::open(&target_pbo)?)?,
                        target_pbo.with_extension(format!("{authority}.bisign")),
                        addons_key.clone(),
                    )
                }
                Location::Optionals => {
                    let (mut target_pbo, key, authority) =
                        if ctx.config().hemtt().build().optional_mod_folders() {
                            let authority = get_authority(ctx, Some(&pbo_name))?;
                            let key = BIPrivateKey::generate(1024, &authority)?;
                            let pubkey = key.to_public_key();
                            let mod_root = ctx
                                .hemtt_folder()
                                .join("optionals")
                                .join(format!("@{pbo_name}"));
                            create_dir_all(mod_root.join("keys"))?;
                            pubkey.write(&mut File::create(
                                mod_root.join("keys").join(format!("{authority}.bikey")),
                            )?)?;
                            (mod_root.join("addons").join(pbo_name), key, authority)
                        } else {
                            (
                                ctx.hemtt_folder()
                                    .join(addon.location().to_string())
                                    .join(pbo_name),
                                addons_key.clone(),
                                authority.clone(),
                            )
                        };
                    target_pbo.set_extension("pbo");
                    (
                        ReadablePbo::from(File::open(&target_pbo)?)?,
                        target_pbo.with_extension(format!("{authority}.bisign")),
                        key,
                    )
                }
            };
            println!("signing `{}`", sig_location.display());
            let sig = key.sign(&mut pbo, ctx.config().signing().version())?;
            sig.write(&mut File::create(sig_location)?)?;
            Result::<(), Error>::Ok(())
        })?;
        Ok(())
    }
}

fn get_authority(ctx: &crate::context::Context, suffix: Option<&str>) -> Result<String, Error> {
    let mut authority = format!(
        "{}_{}",
        ctx.config().signing().authority().map_or_else(
            || ctx.config().prefix().to_string(),
            std::string::ToString::to_string
        ),
        ctx.config().version().get()?
    );
    if let Some(suffix) = suffix {
        authority.push_str(&format!("_{suffix}"));
    }
    Ok(authority)
}
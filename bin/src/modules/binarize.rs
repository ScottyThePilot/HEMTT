use std::{
    fs::create_dir_all,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicI16, Ordering},
};

use hemtt_pbo::{prefix::FILES, Prefix};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

use super::Module;
use crate::{context::Context, error::Error};

#[derive(Default)]
pub struct Binarize {
    command: Option<String>,
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "Binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, _ctx: &Context) -> Result<(), Error> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let path: String = hkcu
            .open_subkey("Software\\Bohemia Interactive\\binarize")?
            .get_value("path")?;
        let path = PathBuf::from(path).join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn init(&mut self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        if self.command.is_none() {
            if cfg!(target_os = "linux") {
                warn!("Binarize is not available on non-Windows platforms.");
            } else {
                warn!("Binarize was not found in the system registery.");
            }
            return Ok(());
        }

        let mut targets = Vec::new();

        let counter = AtomicI16::new(0);
        let tmp_source = ctx.tmp().join("source");
        let tmp_out = ctx.tmp().join("output");
        for addon in ctx.addons() {
            for entry in ctx.vfs().join(addon.folder()).unwrap().walk_dir().unwrap() {
                let entry = entry.unwrap();
                if entry.metadata().unwrap().file_type == VfsFileType::File
                    && ["rtm", "p3d"].contains(&entry.extension().unwrap_or_default().as_str())
                {
                    if let Some(config) = addon.config() {
                        if config
                            .no_bin(&addon.folder())
                            .unwrap()
                            .contains(&PathBuf::from(entry.as_str().trim_start_matches('/')))
                        {
                            debug!("skipping binarization of {}", entry.as_str());
                            continue;
                        }
                    }

                    // skip OLOD & BMTR files as they are already binarized
                    let mut buf = [0; 4];
                    entry.open_file().unwrap().read_exact(&mut buf).unwrap();
                    if buf == [0x4F, 0x4C, 0x4F, 0x44] || buf == [0x42, 0x4D, 0x54, 0x52] {
                        continue;
                    }

                    let addon_root = PathBuf::from(entry.as_str())
                        .components()
                        .take(3)
                        .collect::<PathBuf>();
                    let mut tmp_sourced = None;
                    let mut tmp_outed = None;
                    for file in FILES {
                        let prefix_path = ctx
                            .vfs()
                            .join(
                                addon_root
                                    .join(file)
                                    .to_str()
                                    .unwrap()
                                    .replace('\\', "/")
                                    .trim_start_matches('/'),
                            )
                            .unwrap();
                        if prefix_path.exists().unwrap() {
                            let prefix = Prefix::new(
                                &prefix_path.read_to_string().unwrap(),
                                ctx.config().hemtt().pbo_prefix_allow_leading_slash(),
                            )
                            .unwrap()
                            .into_inner();
                            tmp_sourced = Some(tmp_source.join(prefix.trim_start_matches('\\')));
                            tmp_outed =
                                Some(tmp_out.join(entry.parent().as_str().trim_start_matches('/')));
                            break;
                        }
                    }
                    let tmp_sourced = tmp_sourced.unwrap().join(
                        entry
                            .as_str()
                            .trim_start_matches('/')
                            .trim_start_matches(&addon.folder().to_string())
                            .trim_start_matches('/')
                            .trim_end_matches(&entry.filename())
                            .replace('/', "\\"),
                    );
                    let tmp_outed = tmp_outed.unwrap();

                    targets.push(BinarizeTarget {
                        source: tmp_sourced
                            .to_str()
                            .unwrap()
                            .trim_start_matches('/')
                            .trim_start_matches(&addon.folder())
                            .to_owned(),
                        output: tmp_outed.to_str().unwrap().to_owned(),
                        entry: entry.filename().trim_start_matches('/').to_owned(),
                    });
                }
            }
        }

        targets.par_iter().for_each(|target| {
            debug!("binarizing {}", target.entry);
            create_dir_all(&target.output).unwrap();
            let exe = self.command.as_ref().unwrap();
            let mut cmd = Command::new(exe);
            cmd.args([
                "-norecurse",
                "-always",
                "-silent",
                "-maxProcesses=0",
                &target.source,
                &target.output,
                &target.entry,
            ])
            .current_dir(&tmp_source);
            trace!("{:?}", cmd);
            let output = cmd.output().unwrap();
            assert!(
                output.status.success(),
                "binarize failed with code {:?}",
                output.status.code().unwrap_or(-1)
            );
            if !PathBuf::from(&target.output).join(&target.entry).exists() {
                error!(
                    "No output file for {}, it likely failed to binarize",
                    target.entry
                );
            }
            counter.fetch_add(1, Ordering::Relaxed);
        });

        info!("Binarized {} files", counter.load(Ordering::Relaxed));
        Ok(())
    }
}

struct BinarizeTarget {
    source: String,
    output: String,
    entry: String,
}

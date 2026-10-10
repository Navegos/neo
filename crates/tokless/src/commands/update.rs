use crate::commands::helpers::*;
use crate::commands::init::run_init;
use crate::commands::selfupdate::maybe_self_update;
use crate::core::list_tools;
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::progress::*;
use crate::util::version::semver_compare;
use crate::util::versions::gather_versions_force;

pub fn run_update(mut opts: InitOptions) -> i32 {
    maybe_self_update(&opts);

    cmd_header("update", "refresh tools to latest");

    if opts.dry_run {
        L.raw(&format!("  {} Dry run — probe only, no installs.", C.cyan(Sym::INFO)));
        L.raw("");
    }

    let versions = gather_versions_force();
    let mut changed = Vec::new();

    tree_top("Versions");
    for t in list_tools() {
        if t.instruction_only {
            continue;
        }
        let info = versions.get(&t.id);
        let name = paint_name(&pad_end(&t.id, 14));

        let installed = if let Some(info) = info {
            if let Some(ref inst) = info.installed {
                paint_ver(&pad_end(&format!("v{}", inst), 10))
            } else {
                C.dim(&pad_end("not on PATH", 10))
            }
        } else {
            C.dim(&pad_end("not on PATH", 10))
        };

        let latest = if let Some(info) = info {
            if let Some(ref lat) = info.latest {
                paint_ver(&pad_end(&format!("v{}", lat), 10))
            } else {
                C.dim(&pad_end("?", 10))
            }
        } else {
            C.dim(&pad_end("?", 10))
        };

        let mut mark = C.gray(Sym::BULLET);
        let mut suffix = C.dim(" (latest unknown)");

        if let Some(info) = info {
            if let (Some(inst), Some(lat)) = (&info.installed, &info.latest) {
                if semver_compare(inst, lat) < 0 {
                    mark = C.yellow("↑");
                    suffix = C.yellow(" → upgrade");
                    changed.push(t.id.clone());
                } else {
                    mark = C.green(Sym::CHECK);
                    suffix = C.green(" (up to date)");
                }
            } else if info.installed.is_none() && info.latest.is_some() {
                mark = C.yellow("+");
                suffix = C.yellow(" → install");
                changed.push(t.id.clone());
            }
        }

        tree_leaf(&format!("{} {} {} {} {}{}", mark, name, installed, paint_arrow(), latest, suffix));
    }
    tree_close();

    if opts.dry_run {
        if !changed.is_empty() {
            tree_status(&[status_info(&format!("{} {}", C.gray("Would upgrade: "), C.bold(&paint_ver(&join_comma(&changed)))))]);
        } else {
            tree_status(&[status_ok("Everything up to date.")]);
        }
        print_repo_footer(true);
        L.raw("");
        return 0;
    }

    if changed.is_empty() {
        tree_status(&[status_ok("Everything up to date.")]);
        print_repo_footer(true);
        L.raw("");
        return 0;
    }

    opts.tools = Some(changed);
    opts.upgrade = true;
    run_init(opts)
}

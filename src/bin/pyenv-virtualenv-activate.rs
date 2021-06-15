use pyenv_virtualenv_activate::{handle_result, pyenv_sh_activate, pyenv_sh_deactivate, CommonOpt};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(flatten)]
    opt: CommonOpt,
    #[structopt(long)]
    unset: bool,
}

fn main() {
    let opt = Opt::from_args();

    let res = if opt.unset {
        pyenv_sh_deactivate(opt.opt.force, opt.opt.quiet)
    } else {
        pyenv_sh_activate(opt.opt.version, opt.opt.force, opt.opt.quiet)
    };
    handle_result(res, opt.opt.quiet);
}

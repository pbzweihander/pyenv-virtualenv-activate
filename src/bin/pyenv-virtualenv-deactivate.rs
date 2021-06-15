use pyenv_virtualenv_activate::{handle_result, pyenv_sh_deactivate, CommonOpt};
use structopt::StructOpt;

fn main() {
    let opt = CommonOpt::from_args();

    handle_result(pyenv_sh_deactivate(opt.force, opt.quiet), opt.quiet);
}

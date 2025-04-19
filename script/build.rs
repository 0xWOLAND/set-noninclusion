use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    let args = BuildArgs {
        binaries: vec![
            "noninclusion-program".to_string(),
            "fold-program".to_string(),
        ],
        ..Default::default()
    };
    build_program_with_args("../program", args);
}

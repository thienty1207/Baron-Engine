use baron_core::{phase, product_name};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    println!("{} {}", product_name(), phase());
    println!("Phase 0 skeleton only. See docs/specs and docs/roadmap before implementation.");
}

fn print_help() {
    println!("Baron Engine");
    println!();
    println!("Phase 0 command preview:");
    println!("  baron survey");
    println!("  baron init --codex");
    println!("  baron init --claude");
    println!("  baron init --agent");
    println!("  baron context --codex");
    println!("  baron context --claude");
    println!("  baron context --agent");
    println!("  baron recall \"<query>\"");
    println!("  baron memory status");
    println!("  baron plan status");
    println!("  baron harness status");
    println!("  baron trace score");
}

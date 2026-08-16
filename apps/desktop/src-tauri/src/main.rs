fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--runner") {
        let value_after = |flag: &str| {
            arguments
                .iter()
                .position(|argument| argument == flag)
                .and_then(|index| arguments.get(index + 1))
                .map(std::path::PathBuf::from)
        };
        let socket = value_after("--socket").expect("--runner requires --socket");
        let database = value_after("--database").expect("--runner requires --database");
        agent_pipeline_runner::serve(&socket, &database).expect("local runner failed");
        return;
    }
    agent_pipeline_desktop_lib::run();
}

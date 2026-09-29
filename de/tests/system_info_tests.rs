use rusty_de::service::system_info::SystemInfo;
use std::sync::OnceLock;

// Ensure GTK is initialized once per test binary
static GTK_INIT: OnceLock<()> = OnceLock::new();

fn ensure_gtk_init() {
    GTK_INIT.get_or_init(|| {
        gtk4::init().expect("Failed to initialize GTK");
    });
}

#[test]
fn test_system_info_singleton_pattern() {
    ensure_gtk_init();
    let instance1 = SystemInfo::instance();
    let instance2 = SystemInfo::instance();

    assert_eq!(
        instance1.cpu_usage(),
        instance2.cpu_usage(),
        "Singleton should return same instance"
    );
}

#[test]
fn test_cpu_usage_is_valid_percentage() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let cpu = system_info.cpu_usage();
    assert!(cpu >= 0.0, "CPU usage should be >= 0, got {}", cpu);
    assert!(cpu <= 100.0, "CPU usage should be <= 100, got {}", cpu);
}

#[test]
fn test_memory_usage_not_empty() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let memory = system_info.memory_usage();
    assert!(!memory.is_empty(), "Memory usage should not be empty");
    assert!(
        memory.contains("GB"),
        "Memory usage should contain 'GB', got: {}",
        memory
    );
}

#[test]
fn test_swap_usage_has_content() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let swap = system_info.swap_usage();
    assert!(!swap.is_empty(), "Swap usage should not be empty");
    // Swap might be "Not available" or contain GB
    assert!(
        swap.contains("GB") || swap.contains("Not available"),
        "Swap usage should contain 'GB' or 'Not available', got: {}",
        swap
    );
}

#[test]
fn test_disk_usage_has_content() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let disk = system_info.disk_usage();
    assert!(!disk.is_empty(), "Disk usage should not be empty");
    // Disk might be "Not available" or contain a percentage
    assert!(
        disk.contains("%") || disk.contains("Not available"),
        "Disk usage should contain '%' or 'Not available', got: {}",
        disk
    );
}

#[test]
fn test_top_processes_returns_list() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let processes = system_info.top_processes();
    assert!(
        processes.len() <= 10,
        "Should return at most 10 processes, got {}",
        processes.len()
    );
}

#[test]
fn test_process_info_has_required_fields() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let processes = system_info.top_processes();
    if let Some(process) = processes.first() {
        assert!(!process.pid.is_empty(), "Process PID should not be empty");
        assert!(!process.cpu.is_empty(), "Process CPU should not be empty");
        assert!(!process.mem.is_empty(), "Process MEM should not be empty");
        assert!(
            !process.command.is_empty(),
            "Process command should not be empty"
        );
        assert!(
            !process.full_command.is_empty(),
            "Process full_command should not be empty"
        );
    }
}

#[test]
fn test_command_truncation() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let processes = system_info.top_processes();
    for process in processes {
        assert!(
            process.command.len() <= 40,
            "Command should be truncated to 40 chars or less, got {} chars",
            process.command.len()
        );

        // If command is truncated, it should end with "..."
        if process.full_command.len() > 40 {
            assert!(
                process.command.ends_with("..."),
                "Truncated command should end with '...'"
            );
        }
    }
}

#[test]
fn test_refresh_updates_data() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let _cpu_before = system_info.cpu_usage();

    // Refresh the data
    system_info.refresh();

    let cpu_after = system_info.cpu_usage();

    // CPU should still be a valid percentage
    assert!(cpu_after >= 0.0, "CPU usage should be >= 0 after refresh");
    assert!(
        cpu_after <= 100.0,
        "CPU usage should be <= 100 after refresh"
    );
}

#[test]
fn test_memory_format() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let memory = system_info.memory_usage();

    // Should match format: "X.X GB / Y.Y GB (ZZ%)"
    assert!(memory.contains('/'), "Memory should contain '/'");
    assert!(memory.contains('('), "Memory should contain '('");
    assert!(memory.contains(')'), "Memory should contain ')'");
    assert!(memory.contains('%'), "Memory should contain '%'");
}

#[test]
fn test_disk_format() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let disk = system_info.disk_usage();

    if disk != "Not available" {
        // Should match format: "X / Y (ZZ%)"
        assert!(disk.contains('/'), "Disk should contain '/'");
        assert!(disk.contains('('), "Disk should contain '('");
        assert!(disk.contains(')'), "Disk should contain ')'");
        assert!(disk.contains('%'), "Disk should contain '%'");
    }
}

#[test]
fn test_full_command_contains_command() {
    ensure_gtk_init();
    let system_info = SystemInfo::instance();

    let processes = system_info.top_processes();
    for process in processes {
        // The displayed command should be a prefix or equal to full_command
        if !process.command.ends_with("...") {
            assert_eq!(
                process.command, process.full_command,
                "Non-truncated command should equal full command"
            );
        } else {
            // Truncated command (without "...") should be prefix of full_command
            let command_without_ellipsis = process.command.trim_end_matches("...");
            assert!(
                process.full_command.starts_with(command_without_ellipsis),
                "Truncated command should be prefix of full command"
            );
        }
    }
}

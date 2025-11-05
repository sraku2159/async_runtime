use async_runtime::Engine;
use async_runtime::engine::block_on;
use async_runtime::engine::schedule::fifo::Fifo;
use std::time::Duration;

fn main() {
    println!("=== Complex Task Dependencies Example ===\n");
    println!("Simulating a web service with multiple dependent API calls\n");

    let mut engine = Engine::new(4, |receiver| Box::new(Fifo::new(receiver)));

    // Simulate fetching user data
    println!("[1] Fetching user profile...");
    let user_profile = engine.reserve(
        async {
            // Simulate network delay
            println!("  -> User profile API called");
            std::thread::sleep(Duration::from_millis(100));
            ("Alice".to_string(), 42)
        },
        None,
    );

    // Simulate fetching user's friends list (depends on user ID)
    println!("[2] Fetching friends list...");
    let friends_list = engine.reserve(
        async move {
            let (username, _user_id) = block_on(user_profile);
            println!("  -> Friends API called for user: {}", username);
            std::thread::sleep(Duration::from_millis(150));
            vec![
                format!("{}'s friend 1", username),
                format!("{}'s friend 2", username),
                format!("{}'s friend 3", username),
            ]
        },
        None,
    );

    // Simulate fetching user's posts
    println!("[3] Fetching user posts...");
    let user_posts = engine.reserve(
        async {
            println!("  -> Posts API called");
            std::thread::sleep(Duration::from_millis(120));
            vec!["Post 1", "Post 2", "Post 3"]
        },
        None,
    );

    // Process friends and posts together
    println!("[4] Processing aggregated data...");
    let aggregated = engine.reserve(
        async move {
            let friends = block_on(friends_list);
            let posts = block_on(user_posts);

            println!(
                "  -> Aggregating {} friends and {} posts",
                friends.len(),
                posts.len()
            );

            // Simulate some processing
            let friend_count = friends.len();
            let post_count = posts.len();

            (friend_count, post_count)
        },
        None,
    );

    // Fetch notifications (independent task)
    println!("[5] Fetching notifications...");
    let notifications = engine.reserve(
        async {
            println!("  -> Notifications API called");
            std::thread::sleep(Duration::from_millis(80));
            5 // number of notifications
        },
        None,
    );

    // Final dashboard assembly
    println!("[6] Assembling dashboard...");
    let dashboard = engine.reserve(
        async move {
            let (friend_count, post_count) = block_on(aggregated);
            let notif_count = block_on(notifications);

            println!("\n=== Dashboard Ready ===");
            println!("Friends: {}", friend_count);
            println!("Posts: {}", post_count);
            println!("Notifications: {}", notif_count);
            println!("======================\n");

            (friend_count, post_count, notif_count)
        },
        None,
    );

    println!("\nWaiting for dashboard to be ready...\n");
    let result = block_on(dashboard);

    println!("Dashboard loaded successfully!");
    println!(
        "Total data points: {} friends, {} posts, {} notifications",
        result.0, result.1, result.2
    );

    println!("\nShutting down engine...");
    engine.graceful_shutdown();
    println!("Done!");
}

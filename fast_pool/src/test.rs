use crate::*;

#[test]
fn join() -> std::io::Result<()> {
    let pool = ThreadPoolBuilder::new().build()?;
    pool.shutdown();
    Ok(())
}

#[test]
fn wait_task() -> std::io::Result<()> {
    let pool = ThreadPoolBuilder::new().build()?;
    let handle = pool.spawn(|| {
        println!("Hello world");
        "hello world"
    });
    let value = handle.wait();
    println!("Value: {:?}", value);

    Ok(())
}

#[test]
#[should_panic]
fn spawn_shutdown() {
    let pool = ThreadPoolBuilder::new().build().unwrap();
    let handle = pool.handle();
    pool.shutdown();
    handle.spawn(|| {
        println!("Hello world");
        "hello world"
    });
}

#[test]
#[should_panic]
fn get_uninitialized() {
    let handle = Handle::current().spawn(|| {
        println!("Hello world");
        "hello world"
    });
    let value = handle.wait();
    println!("Value: {:?}", value);
}

#[test]
fn run_custom() -> std::io::Result<()> {
    struct Test;
    impl Task for Test {
        type Output = &'static str;
        
        fn run(self) -> &'static str {
            println!("Running test");
            std::thread::sleep(std::time::Duration::from_secs(5));
            "Test ran!"
        }
    }

    let pool = ThreadPoolBuilder::new().build()?;
    println!("{:?}", pool.spawn(Test).wait());
    Ok(())
}

#[tokio::test]
async fn wait_async() -> std::io::Result<()> {
    let pool = ThreadPoolBuilder::new().build()?;

    pool.spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }).await.unwrap();

    println!("Done");

    Ok(())
}

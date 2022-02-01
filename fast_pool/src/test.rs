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
            "Test ran!"
        }
    }

    let pool = ThreadPoolBuilder::new().build()?;
    println!("{:?}", pool.spawn(Test).wait());
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn print_future() -> std::io::Result<()> {
    let pool = ThreadPoolBuilder::new().build()?;
    let fut = pool.spawn_async(async move {
        println!("Hello from a task!");
        "Hello world"
    });
    print!("{:?}", fut.await);
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn tokio_sleep() -> std::io::Result<()> {
    let pool = ThreadPoolBuilder::new().build()?;
    let handle = tokio::runtime::Handle::current();
    let handle = pool.spawn_async(async move {
        let _guard = handle.enter();
        println!("Sleeping");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("Slept");
        drop(_guard);
    });

    if let Err(why) = handle.await {
        println!("{:#?}", why);
    }
    Ok(())
}

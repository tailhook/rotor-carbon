extern crate argparse;
extern crate rotor;
extern crate rotor_carbon;
extern crate rand;

struct Context;

use std::thread;
use std::time::Duration;

use rand::{thread_rng, Rng};
use argparse::{ArgumentParser, Store};
use rotor_carbon::connect_ip;


fn main() {
    let mut host = "127.0.0.1".to_string();
    let mut port = 2003u16;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Submits random data to the carbon (so you can view in graphite)
            ");
        ap.refer(&mut host).add_argument("host", Store, "
            Host to connect to. Name resolution is done on start only.");
        ap.refer(&mut port).add_argument("port", Store, "
            Port to connect to. Default is 2003 which is the port of
            graphite text protocol example.");
        ap.parse_args_or_exit();
    }

    let mut sink_cell = None;
    let mut loop_creator = rotor::Loop::new(
        &rotor::Config::new()).unwrap();
    loop_creator.add_machine_with(|scope| {
        let (fsm, sink) = connect_ip(
            format!("{}:{}", host, port).parse().unwrap(),
            scope).unwrap();
        sink_cell = Some(sink);
        Ok(fsm)
    }).unwrap();
    let sink = sink_cell.unwrap();

    // We create a loop in the thread. It's simpler to use for demo.
    // But it's perfectly okay to add rotor-carbon thing to your normal
    // event loop
    thread::spawn(move || {
        loop_creator.run(Context).unwrap();
    });

    let mut rng = thread_rng();
    loop {
        thread::sleep(Duration::from_secs(1));
        {
            let mut sender = sink.sender();
            sender.add_int("test.localhost.random.value1",
                rng.gen_range(0, 200));
            sender.add_int("test.localhost.random.value2",
                rng.gen_range(100, 500));
        }
    }
}

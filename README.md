# rxrs - Reactive Extensions for Rust


# 🌱 This Project is currently in its early stage...
# 🦀 Contributions Are Welcome!

# Example

```rust
`(Rust Nightly 1.25+)`

#[test]
fn timer()
{
    println!("cur thread {:?}", thread::current().id());

    rxfac::timer(0, Some(100), NewThreadScheduler::get())
        .skip(3)
        .filter(|i| i % 2 == 0)
        .take(3)
        .map(|v| format!("-{}-", v))
        .observe_on(NewThreadScheduler::get())
        .subf(
            |v| println!("{} on {:?}", v, thread::current().id()),
            (),
            || println!("complete on {:?}", thread::current().id())
        );

    thread::sleep(::std::time::Duration::from_millis(1000));
}
```
Output:
```bash
cur thread ThreadId(1)
-4- on ThreadId(2)
-6- on ThreadId(2)
-8- on ThreadId(2)
```

# File Structure
```
src
├── behaviour_subject.rs
├── fac
│   ├── create.rs
│   ├── mod.rs
│   └── timer.rs
├── lib.rs
├── observable.rs
├── op
│   ├── filter.rs
│   ├── map.rs
│   ├── mod.rs
│   ├── observe_on.rs
│   ├── skip.rs
│   ├── sub_on.rs
│   ├── take.rs
│   └── take_until.rs
├── scheduler.rs
├── subject.rs
├── subscriber.rs
├── unsub_ref.rs
└── util
    ├── arc_cell.rs
    ├── atomic_option.rs
    └── mod.rs
```

# TODO
- more operators
- `Scheduler`s
- docs
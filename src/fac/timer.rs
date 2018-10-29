use std::time::Duration;
use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::*;
use crate::util::any_send_sync::AnySendSync;
use crate::act::WrapAct;
use std::cell::Cell;
use std::cell::RefCell;

pub struct Timer<SS: YesNo, Sch: SchedulerPeriodic<SS>>
{
    period: Duration,
    scheduler: Arc<Sch>,
    PhantomData: PhantomData<SS>
}

impl<SS:YesNo, Sch: SchedulerPeriodic<SS>> Timer<SS, Sch>
{
    pub fn new(period: Duration, scheduler: Sch) -> Self
    {
        Timer{ period, scheduler: Arc::new(scheduler), PhantomData }
    }
}


impl<SS:YesNo, Sch: SchedulerPeriodic<SS>+'static>
Observable<'static, SS, Val<usize>>
for Timer<SS, Sch>
{
    fn sub(&self, next: impl ActNext<'static, SS, Val<usize>>, ec: impl ActEc<'static, SS>) -> Unsub<'static, SS>
    {
        let count = AtomicUsize::new(0);
        //hack: prevent sch being dropped when Timer is dropped
        let sch = self.scheduler.clone();

        self.scheduler.schedule_periodic(self.period, unsafe { WrapAct::new(move |unsub: Ref<Unsub<'static, SS>>|{
            sch.as_ref();

            if !next.stopped() {
                next.call(count.fetch_add(1, Ordering::Relaxed));
            }

            if next.stopped() { unsub.as_ref().unsub(); }
        })})
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, SS, Val<usize>>>, ec: Box<ActEcBox<'static, SS>>) -> Unsub<'static, SS>
    { self.sub(next, ec) }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::time::Duration;
    use std::sync::Arc;
    use std::rc::Rc;
    use std::cell::Cell;
    use std::sync::atomic::*;
    use std::sync::Mutex;
    use std::cell::RefCell;

    #[test]
    fn smoke()
    {
        let (n, n1) = Arc::new(AtomicUsize::new(0)).clones();
        let t = Timer::new(Duration::from_millis(33), NewThreadScheduler::new(Arc::new(DefaultThreadFac)));

        t.take(10).sub(move |v| { n.store(v, Ordering::SeqCst); }, ());
        assert_ne!(n1.load(Ordering::SeqCst), 9);


        ::std::thread::sleep_ms(1000);
        assert_eq!(n1.load(Ordering::SeqCst), 9);
    }

    #[test]
    fn ops()
    {
        let timer: impl Observable<YES, Val<usize>> = Timer::new(Duration::from_millis(10), NewThreadScheduler::new(Arc::new(DefaultThreadFac)));

        let (out, out1, out3) = Arc::new(Mutex::new(String::new())).clones();

        timer.filter(|v: &_| v % 2 == 0 ).take(5).map(|v| format!("{}", v)).sub(
            move |v: String| { out.lock().unwrap().push_str(&*v); },
            move |e: Option<&_>| out3.lock().unwrap().push_str("ok")
        );

        ::std::thread::sleep_ms(1000);

        assert_eq!(out1.lock().unwrap().as_str(), "02468ok");
    }

    #[test]
    fn multiple_times()
    {
        let n = Arc::new(Mutex::new(0));
        let t = Arc::new(Timer::new(Duration::from_millis(10), NewThreadScheduler::new(Arc::new(DefaultThreadFac))));

        for i in 0..5 {
            let (n1, n2) = n.clone().clones();
            let t = t.clone();
            t.take(1).sub(
                move |v| *n1.lock().unwrap() += i,
                move |e:Option<&_>| *n2.lock().unwrap() += 1
            );
        }

        ::std::thread::sleep_ms(500);
        assert_eq!(*n.lock().unwrap(), 10 + 5);
    }

//    #[test]
//    fn as_until_sig()
//    {
//        let (n, n1, n2) = Arc::new(Mutex::new(0)).clones();
//        let (s, s1) = Arc::new(Subject::<YES, i32>::new()).clones();
//        let t = Timer::new(Duration::from_millis(100), NewThreadScheduler::new(Arc::new(DefaultThreadFac)));
//
//        s.until(t).sub(
//            move |v| *n1.lock().unwrap() += v ,
//            move |e: Option<_>| *n2.lock().unwrap() += 100
//        );
//
//        s1.next(1);
//        assert_eq!(*n.lock().unwrap(), 1);
//
//        s1.next(2);
//        assert_eq!(*n.lock().unwrap(), 3);
//
//        ::std::thread::sleep_ms(10);
//        s1.next(3);
//        assert_eq!(*n.lock().unwrap(), 6);
//
//        ::std::thread::sleep_ms(150);
//        assert_eq!(*n.lock().unwrap(), 106);
//
//        s1.next(1234);
//        s1.complete();
//        assert_eq!(*n.lock().unwrap(), 106);
//    }

    #[test]
    fn no()
    {
        let (out, out1, out3) = Rc::new(RefCell::new(String::new())).clones();
        let t = Timer::new(Duration::from_millis(10), CurrentThreadScheduler::new());

        t.filter(|v:&_| v % 2 == 0 ).take(5).map(|v| format!("{}", v)).sub(
            move |v: String| { out.borrow_mut().push_str(&*v); },
            move |e: Option<&_>| out3.borrow_mut().push_str("ok")
        );

        assert_eq!(out1.borrow().as_str(), "02468ok");
    }
}
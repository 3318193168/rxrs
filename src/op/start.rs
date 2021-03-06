use crate::*;
use self::StartOpType::*;
use std::marker::PhantomData;
use std::sync::Mutex;

pub mod StartOpType {
    pub struct ONCE;
    pub struct CLONE;
    pub struct FN;
    pub struct VALREF;
    pub struct REF;
}


pub struct StartOp<Src, V, TYPE>
{
    src: Src,
    v: V,
    PhantomData: PhantomData<TYPE>
}

pub trait ObsStartValOp<'o, V, SS:YesNo> : Sized
{
    fn start_once(self, v: V) -> StartOp<Self, Mutex<Option<V>>, ONCE>;
    fn start(self, v: V) -> StartOp<Self, V, CLONE> where V: Clone+'o;
    fn start_fn<F>(self, f: F) -> StartOp<Self, F, FN> where F: 'o+Fn()->V;
}

pub trait ObsStartRefOp<'o, V:'o, SS:YesNo> : Sized
{
    fn start(self, v: V) -> StartOp<Self, V, VALREF>;
    fn start_ref(self, v: &'o V) -> StartOp<Self, &'o V, REF>;
}

impl<'o, V, SS:YesNo, Src: Observable<'o, SS, Val<V>>>
ObsStartValOp<'o, V, SS>
for Src
{
    fn start_once(self, v: V) -> StartOp<Self, Mutex<Option<V>>, ONCE> { StartOp{ src: self, v: Mutex::new(Some(v)), PhantomData } }
    fn start(self, v: V) -> StartOp<Self, V, CLONE> where V: Clone + 'o { StartOp{ src: self, v, PhantomData} }
    fn start_fn<F>(self, f: F) -> StartOp<Self, F, FN> where F: Fn() -> V + 'o { StartOp{ src: self, v: f, PhantomData} }
}

impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Ref<V>>>
ObsStartRefOp<'o, V, SS>
for Src
{
    fn start(self, v: V) -> StartOp<Self, V, VALREF> { StartOp{ src: self, v, PhantomData} }
    fn start_ref(self, v: &'o V) -> StartOp<Self, &'o V, REF> { StartOp{ src: self, v, PhantomData} }
}

//dyn ===

pub trait DynObsStartValOp<'o, V, SS:YesNo> : Sized
{
    fn start_once(self, v: V) -> DynObservable<'o, 'o, SS, Val<V>>;
    fn start(self, v: V) -> DynObservable<'o, 'o, SS, Val<V>> where V: Clone+'o;
    fn start_fn<F>(self, f: F) -> DynObservable<'o, 'o, SS, Val<V>> where F: 'o+Fn()->V;
}

pub trait DynObsStartRefOp<'o, V:'o, SS:YesNo> : Sized
{
    fn start(self, v: V) -> DynObservable<'o, 'o, SS, Ref<V>>;
    fn start_ref(self, v: &'o V) -> DynObservable<'o, 'o, SS, Ref<V>>;
}

impl<'o, V:'o, SS:YesNo>
DynObsStartValOp<'o, V, SS>
for DynObservable<'o, 'o, SS, Val<V>>
{
    fn start_once(self, v: V) -> DynObservable<'o, 'o, SS, Val<V>> { StartOp{ src: self.src, v: Mutex::new(Some(v)), PhantomData }.into_dyn() }
    fn start(self, v: V) -> DynObservable<'o, 'o, SS, Val<V>> where V: Clone + 'o { StartOp{ src: self.src, v, PhantomData}.into_dyn() }
    fn start_fn<F>(self, f: F) -> DynObservable<'o, 'o, SS, Val<V>> where F: Fn() -> V + 'o { StartOp{ src: self.src, v: f, PhantomData}.into_dyn() }
}

impl<'o, V:'o, SS:YesNo>
DynObsStartRefOp<'o, V, SS>
for DynObservable<'o, 'o, SS, Ref<V>>
{
    fn start(self, v: V) -> DynObservable<'o, 'o, SS, Ref<V>> { StartOp{ src: self.src, v, PhantomData}.into_dyn() }
    fn start_ref(self, v: &'o V) -> DynObservable<'o, 'o, SS, Ref<V>> { StartOp{ src: self.src, v, PhantomData}.into_dyn() }
}




impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Val<V>>+'o>
Observable<'o, SS, Val<V>>
for StartOp<Src, Mutex<Option<V>>, ONCE>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Val<V>>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized {
        if ! next.stopped() {
            if let Some(v) = self.v.lock().unwrap().take() {
                next.call(v);
                if ! next.stopped() {
                    return self.src.subscribe(next, err_or_comp);
                }

            } else {
                err_or_comp.call_once(Some(RxError::simple(None, "value consumed")));
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Val<V>>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}

impl<'o, V:Clone+'o, SS:YesNo, Src: Observable<'o, SS, Val<V>>+'o>
Observable<'o, SS, Val<V>>
for StartOp<Src, V, CLONE>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Val<V>>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized {
        if ! next.stopped() {
            let v = self.v.clone();
            if !next.stopped() {
                next.call(v);
                if ! next.stopped() {
                    return self.src.subscribe(next, err_or_comp);
                }
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Val<V>>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}


impl<'o, V:'o, F:'o+Fn()->V, SS:YesNo, Src: Observable<'o, SS, Val<V>>+'o>
Observable<'o, SS, Val<V>>
for StartOp<Src, F, FN>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Val<V>>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized {
        if ! next.stopped() {
            let v = (self.v)();
            if !next.stopped() {
                next.call(v);
                if ! next.stopped() {
                    return self.src.subscribe(next, err_or_comp);
                }
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Val<V>>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}

impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Ref<V>>+'o>
Observable<'o, SS, Ref<V>>
for StartOp<Src, V, VALREF>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Ref<V>>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized {
        if ! next.stopped() {
            let v = &self.v;
            if !next.stopped() {
                next.call(v);
                if ! next.stopped() {
                    return self.src.subscribe(next, err_or_comp);
                }
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Ref<V>>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}

impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Ref<V>>+'o>
Observable<'o, SS, Ref<V>>
for StartOp<Src, &'o V, REF>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Ref<V>>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized {
        if ! next.stopped() {
            if !next.stopped() {
                next.call(self.v);
                if ! next.stopped() {
                    return self.src.subscribe(next, err_or_comp);
                }
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Ref<V>>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::RefCell;

    #[test]
    fn val_ref()
    {
        let n = RefCell::new(String::new());
        let o = Of::value(1).start(2);
        o.subscribe(|v:&_| n.borrow_mut().push_str(&format!("{}", v)), ());
        assert_eq!(n.borrow().as_str(), "21");
    }

    #[test]
    fn chain()
    {
        let n = RefCell::new(String::new());
        let o = Of::value(1).start(2).start(3).start(4).start(5);
        o.subscribe(|v: &_| n.borrow_mut().push_str(&format!("{}", v)), |_e| n.borrow_mut().push_str("*"));
        assert_eq!(n.borrow().as_str(), "54321*");

        o.subscribe(|v: &_| n.borrow_mut().push_str(&format!("{}", v)), ());
        assert_eq!(n.borrow().as_str(), "54321*54321");
    }

    #[test]
    fn into_dyn()
    {
        let n = RefCell::new(String::new());
        let o: DynObservable<NO, Ref<i32>> = Of::value(1).into_dyn().start(2);
        o.subscribe(|v: &_| n.borrow_mut().push_str(&format!("{}", v)), |_e| n.borrow_mut().push_str("*"));
        assert_eq!(n.borrow().as_str(), "21*");
    }
}
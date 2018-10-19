use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;

pub struct FilterOp<SS, Src, F>
{
    f: Arc<F>,
    src: Src,
    PhantomData: PhantomData<(SS)>
}

pub trait ObservableFilterOp<SS, VBy, EBy, F: Fn(&By<VBy>)->bool> : Sized
{
    fn filter(self, f: F) -> FilterOp<SS, Self, F> { FilterOp{ f: Arc::new(f), src: self, PhantomData} }
}

impl<'o, VBy: RefOrVal, EBy: RefOrVal, Src: Observable<'o, SS, VBy, EBy>, F: Fn(&By<VBy>)->bool+'o, SS:YesNo>
ObservableFilterOp<SS, VBy,EBy, F>
for Src {}


impl<'o, VBy: RefOrVal+'o, EBy:RefOrVal+'o, Src: Observable<'o, NO, VBy, EBy>, F: Fn(&By<VBy>)->bool+'o>
Observable<'o, NO, VBy, EBy>
for FilterOp<NO, Src, F>
{
    fn sub(&self, next: impl ActNext<'o, NO, VBy>, ec: impl ActEc<'o, NO, EBy>) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        let (s1, s2) = Unsub::new().clones();

        s1.added_each(self.src.sub( move |v:By<_>| if f(&v) && !s2.is_done() { next.call(v); }, ec))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, VBy>>, ec: Box<ActEcBox<'o, NO, EBy>>) -> Unsub<'o, NO>
    { self.sub(dyn_to_impl_next(next), dyn_to_impl_ec(ec)) }
}

impl<VBy: RefOrValSSs, EBy: RefOrValSSs, Src: Observable<'static, YES, VBy, EBy>, F: Fn(&By<VBy>)->bool+'static+Send+Sync>
Observable<'static, YES, VBy, EBy>
for FilterOp<YES, Src, F>
{
    fn sub(&self, next: impl ActNext<'static, YES, VBy>, ec: impl ActEc<'static, YES, EBy>) -> Unsub<'static, YES> where Self: Sized
    {
        let (f, next) = (self.f.clone(), sendsync_next(next));
        let (s1, s2) = Unsub::new().clones();

        s1.added_each(self.src.sub(move |v:By<_>| if f(&v) { s2.if_not_done(|| next.call(v)); }, ec))
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, VBy>>, ec: Box<ActEcBox<'static, YES, EBy>>) -> Unsub<'static, YES>
    { self.sub(dyn_to_impl_next_ss(next), dyn_to_impl_ec_ss(ec)) }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);
        let (input, output) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(|v| v.as_ref() % 2 == 0).sub(|v:By<_>| { n.replace(n.get() + *v); }, ());

        for i in 0..10 {
            input.next(i);
        }

        assert_eq!(n.get(), 20);
    }

    #[test]
    fn cb_safe()
    {
        let n = Cell::new(0);
        let (input, output, side_effect) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(move |v| {
            side_effect.complete();
            v.as_ref() % 2 == 0
        }).sub(|v:By<_>| { n.replace(n.get() + *v); }, ());

        for i in 0..10 {
            input.next(i);
        }

        assert_eq!(n.get(), 0);
    }

    #[test]
    fn should_complete()
    {
        let (n1, n2, n3) = Rc::new(Cell::new(0)).clones();
        let (input, output) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(move |_| true).sub(
            move |v:By<_>        | {  n1.replace(n1.get() + *v); },
            move |_:Option<By<_>>| {  n2.replace(n2.get() + 1);  });

        input.next(1);
        input.next(2);

        assert_eq!(n3.get(), 3);

        input.complete();

        assert_eq!(n3.get(), 4);
    }
}

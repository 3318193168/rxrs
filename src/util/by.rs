use std::marker::PhantomData;
use std::ops::Deref;

pub unsafe trait RefOrVal {
    type V: Sized;
    type RAW: Sized;

    fn as_ref(&self) -> &Self::RAW;
    fn into_v(self) -> Self::V;
    fn from_v(v: Self::V) -> Self;
}

pub trait RefOrValSSs: RefOrVal+Send+Sync+'static {}
impl<T: RefOrVal+Send+Sync+'static> RefOrValSSs for T {}

pub struct Ref<V>(*const V);
pub struct Val<V>(V);

unsafe impl<V> RefOrVal for Ref<V>
{
    type V = *const V;
    type RAW = V;

    fn as_ref(&self) -> &V { unsafe{ &*self.0 } }
    fn into_v(self) -> Self::V { self.0 }
    fn from_v(v: Self::V) -> Self { Ref(v) }
}
unsafe impl<V> RefOrVal for Val<V>
{
    type V = V;
    type RAW = V;

    fn as_ref(&self) -> &V { &self.0 }
    fn into_v(self) -> Self::V { self.0 }
    fn from_v(v: Self::V) -> Self { Val(v) }

}
unsafe impl RefOrVal for ()
{
    type V = ();
    type RAW = ();

    fn as_ref(&self) -> &() { &self }
    fn into_v(self) -> Self::V { self }
    fn from_v(v: Self::V) -> Self { () }
}

//pub struct By<'a, T: RefOrVal>
//{
//    t: T,
//    PhantomData:PhantomData<&'a ()>
//}
//
////ok?
//unsafe impl<'a, V: Send> Send for Ref<V>{}
//unsafe impl<'a, V: Sync> Sync for Ref<V>{}
//
//impl<'a, V> By<'a, Ref<V>>
//{
//    #[inline(always)]
//    pub fn r(r: &'a V) -> By<'a, Ref<V>> { By{ t: Ref(r), PhantomData } }
//    pub fn as_ref(&self) -> &V { &*self }
//}
//
//impl<'a, V> By<'a, Val<V>>
//{
//    #[inline(always)]
//    pub fn v(v: V) -> By<'a, Val<V>> { By{ t: Val(v), PhantomData } }
//    #[inline(always)]
//    pub fn val(self) -> V { self.t.0 }
//    pub fn as_ref(&self) -> &V { &*self }
//}
//
//impl<'a, V> Deref for By<'a, Ref<V>>
//{
//    type Target = V;
//    #[inline(always)] fn deref(&self) -> &V { unsafe { std::mem::transmute(self.t.0) } }
//}
//
//impl<'a, V> Deref for By<'a, Val<V>>
//{
//    type Target = V;
//    #[inline(always)] fn deref(&self) -> &V { &self.t.0 }
//}

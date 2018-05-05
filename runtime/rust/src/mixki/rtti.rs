use std::any::TypeId;
use std::rc::Rc;

pub trait Downcast<BaseInner>
{
    fn downcast<CInner>(&self) -> Option<(&BaseInner, Rc<CInner>)> where CInner : 'static;
    fn type_id(&self) -> TypeId;
}

pub trait Rtti<EnumType>
{
    fn get_child(&self) -> EnumType;
}

#[macro_export]
macro_rules! impl_mixki_rtti {
	($enum:ident, $root:path $(,$n:ident, $v:path)*) => {        
		pub enum $enum {
			Root,
			$($n (rc::Rc<$v>), )*
		}
		impl rtti::Rtti<$enum> for $root where $root : parser::TypeId
		{
			fn get_child(&self) -> ($enum)
			{
				match (self.type_id)
				{
					$(<$v as parser::TypeId>::TYPE_ID => { return $enum::$n(self.child.clone().downcast().unwrap()) })*
					_ => return $enum::Root
				}
			}
		}
	}
}

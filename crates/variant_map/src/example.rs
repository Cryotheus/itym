use crate::VariantMapKey;
use std::any::{Any, TypeId};
use std::mem::{ManuallyDrop, transmute};
use std::ptr::drop_in_place;

#[derive(Debug)]
pub struct Config {
	password: Vec<u8>,
}

#[derive(Debug)]
pub struct GroupRecord {
	group_name: String,
}

#[derive(Debug, Clone)]
pub struct UserRecord {
	user_name: String,
	display_name: String,
	password_hash: [u8; 32],
	/// In seconds since UNIX epoch.
	creation_date: u64,
	/// In seconds since UNIX epoch.
	last_active: u64,
}

/// `VariantMap` macro here!
// #[derive(Debug, Clone)]
// #[variant_map]
// pub enum Foo {
// 	Title -> String,
// 	Motd -> String,
// 	DebugMode -> (),
// 	User(u64) -> UserRecord,
// 	Group(String) -> GroupRecord,
// 	Config -> Config,
// }

#[derive(Debug, Clone)]
pub enum Foo {
	Title,         // -> String
	Motd,          // -> String
	DebugMode,     // -> ()
	User(u64),     // -> UserRecord
	Group(String), // -> GroupRecord
	Config,        // -> Config
}

unsafe impl VariantMapKey for Foo {
	type Entry = FooEntry;
	type EntryMut<'a> = FooEntryMut<'a>;
	type EntryRef<'a> = FooEntryRef<'a>;
	type Union = FooEntryUnion;

	unsafe fn drop_entry(&self, entry: &mut Self::Union) {
		match self {
			Foo::Title | Foo::Motd => unsafe { drop_in_place(&raw mut *entry.string) },
			Foo::DebugMode => unsafe { drop_in_place(&raw mut *entry.unit) },
			Foo::User(..) => unsafe { drop_in_place(&raw mut *entry.user_record) },
			Foo::Group(..) => unsafe { drop_in_place(&raw mut *entry.group_record) },
			Foo::Config => unsafe { drop_in_place(&raw mut *entry.config) },
		};
	}
}

// The `VariantMap` macro generates:
pub enum FooEntry {
	String(String),
	Unit(()),
	UserRecord(UserRecord),
	GroupRecord(GroupRecord),
	Config(Config),
}

pub enum FooEntryMut<'a> {
	String(&'a mut String),
	Unit(&'a mut ()),
	UserRecord(&'a mut UserRecord),
	GroupRecord(&'a mut GroupRecord),
	Config(&'a mut Config),
}

pub enum FooEntryRef<'a> {
	String(&'a String),
	Unit(&'a ()),
	UserRecord(&'a UserRecord),
	GroupRecord(&'a GroupRecord),
	Config(&'a Config),
}

impl FooEntry {
	pub fn downcast<T: Any>(self) -> Result<T, Self>
	where
		String: Any,
		(): Any,
		UserRecord: Any,
		GroupRecord: Any,
		Config: Any,
	{
		//match `TypeId` of `T` to the variant's contained `TypeId`
		//maybe add a binary search option for large amounts of variants?
		todo!()
	}

	pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T>
	where
		String: Any,
		(): Any,
		UserRecord: Any,
		GroupRecord: Any,
		Config: Any,
	{
		/* ... */
		todo!()
	}

	pub fn downcast_ref<T: Any>(&self) -> Option<&T>
	where
		String: Any,
		(): Any,
		UserRecord: Any,
		GroupRecord: Any,
		Config: Any,
	{
		match (self, TypeId::of::<T>()) {
			(Self::String(a), id) if id == const { TypeId::of::<String>() } => Some(unsafe { transmute::<&String, &T>(a) }),
			(Self::String(..), _) => None,

			(Self::Unit(a), id) if id == const { TypeId::of::<()>() } => Some(unsafe { transmute::<&(), &T>(a) }),
			(Self::Unit(..), _) => None,

			(Self::UserRecord(a), id) if id == const { TypeId::of::<UserRecord>() } => Some(unsafe { transmute::<&UserRecord, &T>(a) }),
			(Self::UserRecord(..), _) => None,

			(Self::GroupRecord(a), id) if id == const { TypeId::of::<GroupRecord>() } => Some(unsafe { transmute::<&GroupRecord, &T>(a) }),
			(Self::GroupRecord(..), _) => None,

			(Self::Config(a), id) if id == const { TypeId::of::<Config>() } => Some(unsafe { transmute::<&Config, &T>(a) }),
			(Self::Config(..), _) => None,
		}
	}
}

impl<'map> FooEntryMut<'map> {
	pub fn downcast<T: Any + Clone>(self) -> Result<T, Self> {
		match self.downcast_ref::<T>() {
			None => Err(self),
			Some(transparent) => Ok(transparent.clone()),
		}
	}

	pub fn downcast_mut<'borrow, T: Any>(&'borrow mut self) -> Option<&'map mut T> {
		/* ... */
		todo!()
	}

	pub fn downcast_ref<'borrow, T: Any>(&'borrow self) -> Option<&'map T> {
		/* ... */
		todo!()
	}

	pub fn to_ref(&'map self) -> FooEntryRef<'map> {
		match self {
			Self::String(a) => FooEntryRef::String(&*a),
			Self::Unit(a) => FooEntryRef::Unit(&*a),
			Self::UserRecord(a) => FooEntryRef::UserRecord(&*a),
			Self::GroupRecord(a) => FooEntryRef::GroupRecord(&*a),
			Self::Config(a) => FooEntryRef::Config(&*a),
		}
	}
}

impl<'map> FooEntryRef<'map> {
	pub fn downcast<T: Any + Clone>(self) -> Result<T, Self> {
		match self.downcast_ref::<T>() {
			None => Err(self),
			Some(transparent) => Ok(transparent.clone()),
		}
	}

	pub fn downcast_ref<'borrow, T: Any>(&'borrow self) -> Option<&'map T> {
		/* ... */
		todo!()
	}
}

pub union FooEntryUnion {
	string: ManuallyDrop<String>,
	unit: ManuallyDrop<()>,
	user_record: ManuallyDrop<UserRecord>,
	group_record: ManuallyDrop<GroupRecord>,
	config: ManuallyDrop<Config>,
}

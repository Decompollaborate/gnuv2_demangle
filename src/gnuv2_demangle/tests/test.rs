/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use gnuv2_demangle::{demangle, DemangleConfig};

use pretty_assertions::assert_eq;

#[test]
fn test_demangling_funcs() {
    static CASES: [(&str, &str); 7] = [
        ("whatever_default__Fcsilx", "whatever_default(char, short, int, long, long long)"),
        ("whatever_signed__FScsilx", "whatever_signed(signed char, short, int, long, long long)"),
        ("whatever_unsigned__FUcUsUiUlx", "whatever_unsigned(unsigned char, unsigned short, unsigned int, unsigned long, long long)"),
        ("whatever_other__Ffdrb", "whatever_other(float, double, long double, bool)"),
        ("whatever_why__Fw", "whatever_why(wchar_t)"),
        ("whatever_pointer__FPcPsPiPlPx", "whatever_pointer(char *, short *, int *, long *, long long *)"),
        ("whatever_const_pointer__FPCcPCsPCiPClPCx", "whatever_const_pointer(char const *, short const *, int const *, long const *, long long const *)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangling_funcs_const_pointer_const() {
    static CASES: [(&str, &str); 5] = [
        (
            "whatever_const_pointer__FPc",
            "whatever_const_pointer(char *)",
        ),
        (
            "whatever_const_pointer__FPCc",
            "whatever_const_pointer(char const *)",
        ),
        (
            "whatever_const_pointer__FCPCc",
            "whatever_const_pointer(char const *const)",
        ),
        (
            "whatever_const_pointer__FCPc",
            "whatever_const_pointer(char *const)",
        ),
        (
            "silly_function__FPCPCPCPCPCc",
            "silly_function(char const *const *const *const *const *)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_func_argless() {
    static CASES: [(&str, &str); 1] = [("argless__Fv", "argless(void)")];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_constructor_destructors() {
    static CASES: [(&str, &str); 5] = [
        ("_$_5tName", "tName::~tName(void)"),
        ("__5tName", "tName::tName(void)"),
        ("__5tNamePCc", "tName::tName(char const *)"),
        ("__5tNameG13tUidUnaligned", "tName::tName(tUidUnaligned)"),
        ("__5tNameRC5tName", "tName::tName(tName const &)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_methods() {
    static CASES: [(&str, &str); 6] = [
        ("SetText__5tNamePCc", "tName::SetText(char const *)"),
        ("SetTextOnly__5tNamePCc", "tName::SetTextOnly(char const *)"),
        (
            "SetUID__5tNameG13tUidUnaligned",
            "tName::SetUID(tUidUnaligned)",
        ),
        ("GetText__C5tName", "tName::GetText(void) const"),
        ("MakeUID__5tNamePCc", "tName::MakeUID(char const *)"),
        (
            "AddActionEventLocator__19ActionButtonManagerP18ActionEventLocatorP12tEntityStore",
            "ActionButtonManager::AddActionEventLocator(ActionEventLocator *, tEntityStore *)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_operators() {
    static CASES: [(&str, &str); 3] = [
        (
            "__eq__C5tNameRC5tName",
            "tName::operator==(tName const &) const",
        ),
        (
            "__ne__C5tNameRC5tName",
            "tName::operator!=(tName const &) const",
        ),
        ("__as__5tNameRC5tName", "tName::operator=(tName const &)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_new_delete() {
    static CASES: [(&str, &str); 6] = [
        (
            "__nw__12AnimatedIconUi",
            "AnimatedIcon::operator new(unsigned int)",
        ),
        (
            "__nw__12AnimatedIconUi19GameMemoryAllocator",
            "AnimatedIcon::operator new(unsigned int, GameMemoryAllocator)",
        ),
        (
            "__dl__12AnimatedIconPv",
            "AnimatedIcon::operator delete(void *)",
        ),
        (
            "__nw__FUi19GameMemoryAllocator",
            "operator new(unsigned int, GameMemoryAllocator)",
        ),
        (
            "__dl__FPv19GameMemoryAllocator",
            "operator delete(void *, GameMemoryAllocator)",
        ),
        (
            "__vn__FUi19GameMemoryAllocator",
            "operator new [](unsigned int, GameMemoryAllocator)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

/*
#[test]
fn test_demangle_namespaced() {
    static CASES: [(&str, &str); 8] = [
        ("__Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::AnimCollisionEntityDSGWrapper(void)"),
        ("_$_Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::~AnimCollisionEntityDSGWrapper(void)"),
        ("UpdateVisibility__Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::UpdateVisibility(void)"),
        ("SetGameObject__Q212ActionButton29AnimCollisionEntityDSGWrapperP22AnimCollisionEntityDSG", "ActionButton::AnimCollisionEntityDSGWrapper::SetGameObject(AnimCollisionEntityDSG *)"),
        ("__as__Q33sim15CollisionObject20CollisionVolumeOwnerRCQ33sim15CollisionObject20CollisionVolumeOwner", "sim::CollisionObject::CollisionVolumeOwner::operator=(sim::CollisionObject::CollisionVolumeOwner const &)"),
        ("__Q33sim16CollisionManager4Area", "sim::CollisionManager::Area::Area(void)"),
        ("Reset__Q33sim16CollisionManager4Area", "sim::CollisionManager::Area::Reset(void)"),
        ("AddPair__Q33sim16CollisionManager4AreaPQ23sim15CollisionObjectT1", "sim::CollisionManager::Area::AddPair(sim::CollisionObject *, sim::CollisionObject *)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}
*/

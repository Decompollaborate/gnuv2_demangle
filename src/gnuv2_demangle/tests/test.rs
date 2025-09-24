/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use gnuv2_demangle::{demangle, DemangleConfig, DemangleError};

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
    static CASES: [(&str, &str); 18] = [
        (
            "__eq__C5tNameRC5tName",
            "tName::operator==(tName const &) const",
        ),
        (
            "__ne__C5tNameRC5tName",
            "tName::operator!=(tName const &) const",
        ),
        ("__as__5tNameRC5tName", "tName::operator=(tName const &)"),
        (
            "__ad__C13tUidUnalignedG13tUidUnaligned",
            "tUidUnaligned::operator&(tUidUnaligned) const",
        ),
        (
            "__aml__13tUidUnalignedUl",
            "tUidUnaligned::operator*=(unsigned long)",
        ),
        (
            "__apl__13PascalCStringRC13PascalCString",
            "PascalCString::operator+=(PascalCString const &)",
        ),
        (
            "__er__C13tUidUnalignedG13tUidUnaligned",
            "tUidUnaligned::operator^(tUidUnaligned) const",
        ),
        ("__ls__7ostreamc", "ostream::operator<<(char)"),
        (
            "__ls__7ostreamP9streambuf",
            "ostream::operator<<(streambuf *)",
        ),
        (
            "__lt__C13tUidUnalignedG13tUidUnaligned",
            "tUidUnaligned::operator<(tUidUnaligned) const",
        ),
        ("__nt__C3ios", "ios::operator!(void) const"),
        (
            "__rs__C13tUidUnalignedi",
            "tUidUnaligned::operator>>(int) const",
        ),
        (
            "__rs__C13tUidUnalignedi",
            "tUidUnaligned::operator>>(int) const",
        ),
        (
            "__vc__C13UnicodeStringi",
            "UnicodeString::operator[](int) const",
        ),
        (
            "__eq__CQ23ods7pointerRCQ23ods7pointer",
            "ods::pointer::operator==(ods::pointer const &) const",
        ),
        (
            "__opPc__13PascalCString",
            "PascalCString::operator char *(void)",
        ),
        ("__opPv__C3ios", "ios::operator void *(void) const"),
        (
            "__vd__9CEditRootPvUi",
            "CEditRoot::operator delete [](void *, unsigned int)",
        ),
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

#[test]
fn test_demangle_namespaced_function() {
    static CASES: [(&str, &str); 4] = [
        // A namespaced function inside a single namespace and a method in a class without namespace are mangled the same, awesome...
        ("a_function__4smolfffi", "smol::a_function(float, float, float, int)"),
        ("a_function__Q26medium3yesfffi", "medium::yes::a_function(float, float, float, int)"),
        ("a_function__Q35silly8my_thing17another_namespacefffi", "silly::my_thing::another_namespace::a_function(float, float, float, int)"),
        ("a_function__Q_18_5silly8my_thing17another_namespace7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stacked7stackedfffi", "silly::my_thing::another_namespace::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::stacked::a_function(float, float, float, int)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_namespaced_methods() {
    static CASES: [(&str, &str); 7] = [
        ("__Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::AnimCollisionEntityDSGWrapper(void)"),
        ("_$_Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::~AnimCollisionEntityDSGWrapper(void)"),
        ("UpdateVisibility__Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::UpdateVisibility(void)"),
        ("SetGameObject__Q212ActionButton29AnimCollisionEntityDSGWrapperP22AnimCollisionEntityDSG", "ActionButton::AnimCollisionEntityDSGWrapper::SetGameObject(AnimCollisionEntityDSG *)"),
        ("Reset__Q33sim16CollisionManager4Area", "sim::CollisionManager::Area::Reset(void)"),
        ("__as__Q33sim15CollisionObject20CollisionVolumeOwnerRCQ33sim15CollisionObject20CollisionVolumeOwner", "sim::CollisionObject::CollisionVolumeOwner::operator=(sim::CollisionObject::CollisionVolumeOwner const &)"),
        ("__Q33sim16CollisionManager4Area", "sim::CollisionManager::Area::Area(void)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_remembered_types() {
    static CASES: [(&str, &str); 7] = [
        ("AddPair__Q33sim16CollisionManager4AreaPQ23sim15CollisionObjectT1", "sim::CollisionManager::Area::AddPair(sim::CollisionObject *, sim::CollisionObject *)"),
        ("CollisionEvent__Q23sim20CollisionSolverAgentPQ23sim8SimStateiT1iRCQ218RadicalMathLibrary6VectorffPPQ23sim15SimulatedObjectT8", "sim::CollisionSolverAgent::CollisionEvent(sim::SimState *, int, sim::SimState *, int, RadicalMathLibrary::Vector const &, float, float, sim::SimulatedObject **, sim::SimulatedObject **)"),
        ("EdgeEdge__Q23sim20SubCollisionDetectorRbRQ218RadicalMathLibrary6VectorT2fT2T2fT2ffPQ23sim15CollisionVolumeT11_", "sim::SubCollisionDetector::EdgeEdge(bool &, RadicalMathLibrary::Vector &, RadicalMathLibrary::Vector &, float, RadicalMathLibrary::Vector &, RadicalMathLibrary::Vector &, float, RadicalMathLibrary::Vector &, float, float, sim::CollisionVolume *, sim::CollisionVolume *)"),
        ("AddPair__Q33sim16CollisionManager4AreaPQ23sim15CollisionObjectT0", "sim::CollisionManager::Area::AddPair(sim::CollisionObject *, sim::CollisionManager::Area)"),
        ("AddPair__FQ33sim16CollisionManager4AreaPQ23sim15CollisionObjectT0", "AddPair(sim::CollisionManager::Area, sim::CollisionObject *, sim::CollisionManager::Area)"),
        ("do_thing__C6StupidG6StupidT1", "Stupid::do_thing(Stupid, Stupid) const"),
        ("do_thing__C6StupidRC6StupidT1", "Stupid::do_thing(Stupid const &, Stupid const &) const"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_const_namespaced_methods() {
    static CASES: [(&str, &str); 3] = [
        (
            "GetAnimController__CQ212ActionButton29AnimCollisionEntityDSGWrapper",
            "ActionButton::AnimCollisionEntityDSGWrapper::GetAnimController(void) const",
        ),
        (
            "GetDrawable__CQ212ActionButton29AnimCollisionEntityDSGWrapper",
            "ActionButton::AnimCollisionEntityDSGWrapper::GetDrawable(void) const",
        ),
        (
            "FindFaceIndexOrder__CQ23sim20SubCollisionDetectorPifff",
            "sim::SubCollisionDetector::FindFaceIndexOrder(int *, float, float, float) const",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_repeater_arg() {
    static CASES: [(&str, &str); 8] = [
        (
            "LinkActionToObjectJoint__19ActionButtonManagerPCcN41",
            "ActionButtonManager::LinkActionToObjectJoint(char const *, char const *, char const *, char const *, char const *)",
        ),
        (
            "repeating__FPCcN24_0",
            "repeating(char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *)",
        ),
        (
            "repeating_2__FPiN24_0PCcN24_25_",
            "repeating_2(int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, int *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *, char const *)",
        ),
        ("do_thing__C6StupidG6StupidN25_1", "Stupid::do_thing(Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid, Stupid) const"),
        ("do_thing__C6StupidR6StupidN25_1", "Stupid::do_thing(Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &, Stupid &) const"),
        ("do_thing__C6StupidRC6StupidN25_1", "Stupid::do_thing(Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &, Stupid const &) const"),
        ("LinkActionToObjectJoint__19ActionButtonManagerPCcN41N15", "ActionButtonManager::LinkActionToObjectJoint(char const *, char const *, char const *, char const *, char const *, char const *)"),
        ("LinkActionToObjectJoint__19ActionButtonManagerPCcN41N15N16iN18", "ActionButtonManager::LinkActionToObjectJoint(char const *, char const *, char const *, char const *, char const *, char const *, char const *, int, int)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_funcs_starting_with_double_underscore() {
    static CASES: [(&str, &str); 3] = [
        ("__overflow__FP9streambufi", "__overflow(streambuf *, int)"),
        ("__default_unexpected__Fv", "__default_unexpected(void)"),
        ("__is_pointer__FPv", "__is_pointer(void *)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_type_info_func() {
    static CASES: [(&str, &str); 20] = [
        (
            "__tf18AssignValueToFloat",
            "AssignValueToFloat type_info function",
        ),
        (
            "__tfQ212ActionButton29AnimCollisionEntityDSGWrapper",
            "ActionButton::AnimCollisionEntityDSGWrapper type_info function",
        ),
        (
            "__tf17__array_type_info",
            "__array_type_info type_info function",
        ),
        ("__tfv", "void type_info function"),
        ("__tfx", "long long type_info function"),
        ("__tfl", "long type_info function"),
        ("__tfi", "int type_info function"),
        ("__tfs", "short type_info function"),
        ("__tfb", "bool type_info function"),
        ("__tfc", "char type_info function"),
        ("__tfw", "wchar_t type_info function"),
        ("__tfr", "long double type_info function"),
        ("__tfd", "double type_info function"),
        ("__tff", "float type_info function"),
        ("__tfUi", "unsigned int type_info function"),
        ("__tfUl", "unsigned long type_info function"),
        ("__tfUx", "unsigned long long type_info function"),
        ("__tfUs", "unsigned short type_info function"),
        ("__tfUc", "unsigned char type_info function"),
        ("__tfSc", "signed char type_info function"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_type_info_node() {
    static CASES: [(&str, &str); 20] = [
        (
            "__ti18AssignValueToFloat",
            "AssignValueToFloat type_info node",
        ),
        (
            "__tiQ212ActionButton29AnimCollisionEntityDSGWrapper",
            "ActionButton::AnimCollisionEntityDSGWrapper type_info node",
        ),
        (
            "__ti17__array_type_info",
            "__array_type_info type_info node",
        ),
        ("__tiv", "void type_info node"),
        ("__tix", "long long type_info node"),
        ("__til", "long type_info node"),
        ("__tii", "int type_info node"),
        ("__tis", "short type_info node"),
        ("__tib", "bool type_info node"),
        ("__tic", "char type_info node"),
        ("__tiw", "wchar_t type_info node"),
        ("__tir", "long double type_info node"),
        ("__tid", "double type_info node"),
        ("__tif", "float type_info node"),
        ("__tiUi", "unsigned int type_info node"),
        ("__tiUl", "unsigned long type_info node"),
        ("__tiUx", "unsigned long long type_info node"),
        ("__tiUs", "unsigned short type_info node"),
        ("__tiUc", "unsigned char type_info node"),
        ("__tiSc", "signed char type_info node"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_ellipsis() {
    static CASES: [(&str, &str); 4] = [
        ("Printf__7ConsolePce", "Console::Printf(char *,...)"),
        (
            "StrPrintf__6choreoPciPCce",
            "choreo::StrPrintf(char *, int, char const *,...)",
        ),
        ("printf__3p3dPCce", "p3d::printf(char const *,...)"),
        ("asdfasdfasdfasdf__Fe", "asdfasdfasdfasdf(...)"),
    ];
    let mut config = DemangleConfig::new();
    config.ellipsis_emit_space_after_comma = false;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_ellipsis_space() {
    static CASES: [(&str, &str); 4] = [
        ("Printf__7ConsolePce", "Console::Printf(char *, ...)"),
        (
            "StrPrintf__6choreoPciPCce",
            "choreo::StrPrintf(char *, int, char const *, ...)",
        ),
        ("printf__3p3dPCce", "p3d::printf(char const *, ...)"),
        ("asdfasdfasdfasdf__Fe", "asdfasdfasdfasdf(...)"),
    ];
    let mut config = DemangleConfig::new();
    config.ellipsis_emit_space_after_comma = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_templated_classes() {
    static CASES: [(&str, &str); 10] = [
        ("begin__t3Map2ZPQ23sim15CollisionObjectZP11DynaPhysDSG", "Map<sim::CollisionObject *, DynaPhysDSG *>::begin(void)"),
        ("find__t8_Rb_tree5ZUiZt4pair2ZCUiZiZt10_Select1st1Zt4pair2ZCUiZiZt4less1ZUiZt9allocator1ZiRCUi", "_Rb_tree<unsigned int, pair<unsigned int const, int>, _Select1st<pair<unsigned int const, int> >, less<unsigned int>, allocator<int> >::find(unsigned int const &)"),
        ("ResizeArray__Q23simt6TArray1ZQ23sim9Collisioni", "sim::TArray<sim::Collision>::ResizeArray(int)"),
        ("Grow__Q23simt6TArray1ZQ23sim9Collision", "sim::TArray<sim::Collision>::Grow(void)"),
        ("Add__Q23simt6TArray1ZQ23sim9CollisionRCQ23sim9Collision", "sim::TArray<sim::Collision>::Add(sim::Collision const &)"),
        ("__tit9AllocPool1Z8FMVEvent", "AllocPool<FMVEvent> type_info node"),
        ("_$_t17ContiguousBinNode1Z11SpatialNode", "ContiguousBinNode<SpatialNode>::~ContiguousBinNode(void)"),
        ("__t17ContiguousBinNode1Z11SpatialNode", "ContiguousBinNode<SpatialNode>::ContiguousBinNode(void)"),
        ("GetSubTreeSize__t17ContiguousBinNode1Z11SpatialNode", "ContiguousBinNode<SpatialNode>::GetSubTreeSize(void)"),
        ("other_function__FPQ215other_namespacet11PlainVector1ZQ215other_namespacet11PlainVector1ZQ215other_namespacet11PlainVector1Zi", "other_function(other_namespace::PlainVector<other_namespace::PlainVector<other_namespace::PlainVector<int> > > *)"),
        // ("a_function__Q25silly9SomeClassRCQ224namespace_for_the_vectort7rVector13ZiZcZbZwZrZsZQ213more_stacking11APlainClassZQ213more_stacking11APlainClassZQ213more_stacking11APlainClassZPvZPiZPCcZRCc", "silly::SomeClass::a_function(const namespace_for_the_vector::rVector<int, char, bool, wchar_t, long double, short, more_stacking::APlainClass, more_stacking::APlainClass, more_stacking::APlainClass, void*, int*, const char *, const char &> &)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_templated_classes_with_numbers() {
    static CASES: [(&str, &str); 11] = [
        (
            "template_with_number__FRt9Something1x39",
            "template_with_number(Something<39> &)",
        ),
        (
            "template_with_number__FRt9Something1xm39",
            "template_with_number(Something<-39> &)",
        ),
        (
            "template_with_unsigned_number__FRt10Something21Ui39",
            "template_with_unsigned_number(Something2<39> &)",
        ),
        (
            "template_with_many_numbers__FRt10Something32Ul39b1",
            "template_with_many_numbers(Something3<39, true> &)",
        ),
        (
            "template_with_numbers_and_types__FRt10Something43Sc39ZiUc32",
            "template_with_numbers_and_types(Something4<''', int, ' '> &)",
        ),
        (
            "_S_oom_malloc__t23__malloc_alloc_template1i0Ui",
            "__malloc_alloc_template<0>::_S_oom_malloc(unsigned int)",
        ),
        (
            "template_with_numbers_and_types__FRt10Something43Sc39ZiPCc7example",
            "template_with_numbers_and_types(Something4<''', int, &example> &)",
        ),
        (
            "actual_function__FRt10SomeVector2Z4NodeR13TestAllocator17AllocatorInstanceG4Node",
            "actual_function(SomeVector<Node, AllocatorInstance> &, Node)",
        ),
        (
            "push__t10SomeVector2Z4NodeR13TestAllocator17AllocatorInstanceG4Node",
            "SomeVector<Node, AllocatorInstance>::push(Node)",
        ),
        (
            "get__Ct10SomeVector2Z4NodeR13TestAllocator17AllocatorInstanceUi",
            "SomeVector<Node, AllocatorInstance>::get(unsigned int) const",
        ),
        (
            "get__Ct3Vec2ZiP9Allocator15GlobalAllocatorUi",
            "Vec<int, &GlobalAllocator>::get(unsigned int) const",
        ),
        // TODO
        // ("wrapper__H1Z4Node_Rt10SomeVector2ZX01R13TestAllocator17AllocatorInstanceX01_RCX01", "Node const & wrapper<Node>(SomeVector<Node, AllocatorInstance> &, Node)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_vtable() {
    static CASES: [(&str, &str); 3] = [
        ("_vt$15ISpatialProxyAA", "ISpatialProxyAA virtual table"),
        (
            "_vt$t11ChangeState1ZQ211CharacterAi4Loco",
            "ChangeState<CharacterAi::Loco> virtual table",
        ),
        (
            "_vt$Q211CharacterAi6GetOut$13EventListener",
            "CharacterAi::GetOut::EventListener virtual table",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_namespaced_globals() {
    static CASES: [(&str, &str); 3] = [
        ("_9TrafficAI$LOOKAHEAD_MIN", "TrafficAI::LOOKAHEAD_MIN"),
        (
            "_Q45First6Second5Third6Fourth$global",
            "First::Second::Third::Fourth::global",
        ),
        (
            "_Q75First6Second5Third6Fourth1A1B1C$funny",
            "First::Second::Third::Fourth::A::B::C::funny",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_function_pointers() {
    static CASES: [(&str, &str); 7] = [
        ("set_terminate__FPFv_v", "set_terminate(void (*)(void))"),
        ("set_unexpected__FPFv_v", "set_unexpected(void (*)(void))"),
        ("pointerness__FPFiGt9Something1x42_t9Something1x39iPFiRt9Something1x42_RCt9Something1x39", "pointerness(Something<39> (*)(int, Something<42>), int, Something<39> const &(*)(int, Something<42> &))"),
        ("pointerness__FPFiGt9Something1x42_t9Something1x39iPFiRt9Something1x42_RCt9Something1x39RFPCce_RQ55First6Second5Third6Fourth1A", "pointerness(Something<39> (*)(int, Something<42>), int, Something<39> const &(*)(int, Something<42> &), First::Second::Third::Fourth::A &(&)(char const *,...))"),
        ("InstallShader__14pddiBaseShaderPCcPFP17pddiRenderContextPCcPCc_P14pddiBaseShaderT1", "pddiBaseShader::InstallShader(char const *, pddiBaseShader *(*)(pddiRenderContext *, char const *, char const *), char const *)"),
        ("set_unexpected__FPPPPFv_v", "set_unexpected(void (****)(void))"),
        ("set_unexpected__FRPPPPPPPFv_v", "set_unexpected(void (*******&)(void))"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_function_pointers_within_function_pointers() {
    static CASES: [(&str, &str); 3] = [
        ("set_terminate__FPFPCc_PFbi_ii", "set_terminate(int (*(*)(char const *))(bool, int), int)"),
        ("set_terminate__FPFv_PFv_viT0PFv_PFPFv_PFv_v_v", "set_terminate(void (*(*)(void))(void), int, void (*(*)(void))(void), void (*(*)(void))(void (*(*)(void))(void)))"),
        (
            "i_hope_nobody_actually_writes_something_like_this__FPFPPFGQ213radPs2CdDrive14DirectoryEntryiPCQ213radPs2CdDrive14DirectoryEntry_Q213radPs2CdDrive14DirectoryEntryPFGQ213radPs2CdDrive14DirectoryEntryiPCQ213radPs2CdDrive14DirectoryEntry_Q213radPs2CdDrive14DirectoryEntryGQ213radPs2CdDrive14DirectoryEntry_PFGQ213radPs2CdDrive14DirectoryEntryiPCQ213radPs2CdDrive14DirectoryEntry_Q213radPs2CdDrive14DirectoryEntryPPFGQ213radPs2CdDrive14DirectoryEntryiPCQ213radPs2CdDrive14DirectoryEntry_Q213radPs2CdDrive14DirectoryEntryT0",
            "i_hope_nobody_actually_writes_something_like_this(radPs2CdDrive::DirectoryEntry (*(*)(radPs2CdDrive::DirectoryEntry (**)(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *), radPs2CdDrive::DirectoryEntry (*)(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *), radPs2CdDrive::DirectoryEntry))(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *), radPs2CdDrive::DirectoryEntry (**)(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *), radPs2CdDrive::DirectoryEntry (*(*)(radPs2CdDrive::DirectoryEntry (**)(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *), radPs2CdDrive::DirectoryEntry (*)(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *), radPs2CdDrive::DirectoryEntry))(radPs2CdDrive::DirectoryEntry, int, radPs2CdDrive::DirectoryEntry const *))",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_global_sym_keyed() {
    static CASES: [(&str, &str); 14] = [
        ("_GLOBAL_$I$_13BootupContext$spInstance", "global constructors keyed to BootupContext::spInstance"),
        ("_GLOBAL_$I$_12ActorManager$ActorRemovalRangeSqr", "global constructors keyed to ActorManager::ActorRemovalRangeSqr"),
        ("_GLOBAL_$D$_12ActorManager$ActorRemovalRangeSqr", "global destructors keyed to ActorManager::ActorRemovalRangeSqr"),
        ("_GLOBAL_$D$_6Action$sMemoryPool", "global destructors keyed to Action::sMemoryPool"),
        ("_GLOBAL_$I$__9FMVPlayer", "global constructors keyed to FMVPlayer::FMVPlayer(void)"),
        ("_GLOBAL_$I$__7ChaseAIP7Vehiclef", "global constructors keyed to ChaseAI::ChaseAI(Vehicle *, float)"),
        ("_GLOBAL_$D$__Q212ActionButton29AnimCollisionEntityDSGWrapper", "global destructors keyed to ActionButton::AnimCollisionEntityDSGWrapper::AnimCollisionEntityDSGWrapper(void)"),
        ("_GLOBAL_$I$GetContext__10ps2Context", "global constructors keyed to ps2Context::GetContext(void)"),
        ("_GLOBAL_$I$_t14radLinkedClass1ZQ25Sound17daSoundPlayerBase$s_pLinkedClassHead", "global constructors keyed to radLinkedClass<Sound::daSoundPlayerBase>::s_pLinkedClassHead"),
        ("_GLOBAL_$D$_t14radLinkedClass1ZQ25Sound17daSoundPlayerBase$s_pLinkedClassHead", "global destructors keyed to radLinkedClass<Sound::daSoundPlayerBase>::s_pLinkedClassHead"),
        ("_GLOBAL_$I$malloc_uncached__Fi", "global constructors keyed to malloc_uncached(int)"),
        ("_GLOBAL_$D$malloc_uncached__Fi", "global destructors keyed to malloc_uncached(int)"),
        ("_GLOBAL_$I$gErrFileName", "global constructors keyed to gErrFileName"),
        ("_GLOBAL_$D$gErrFileName", "global destructors keyed to gErrFileName"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_global_sym_keyed_weird_cases() {
    static CASES: [(&str, &str, &str); 2] = [
        ("_GLOBAL_$I$__Q212ActionButton29AnimCollisionEntityDSGWrapper", "ActionButton::AnimCollisionEntityDSGWrapper::AnimCollisionEntityDSGWrapper(void)", "global constructors keyed to ActionButton::AnimCollisionEntityDSGWrapper::AnimCollisionEntityDSGWrapper(void)"),
        ("_GLOBAL_$I$__Q210Scenegraph10Scenegraph", "Scenegraph::Scenegraph::Scenegraph(void)", "global constructors keyed to Scenegraph::Scenegraph::Scenegraph(void)"),
    ];
    let mut config = DemangleConfig::new();

    config.preserve_namespaced_global_constructor_bug = true;
    for (mangled, demangled, _) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }

    config.preserve_namespaced_global_constructor_bug = false;
    for (mangled, _, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_global_sym_keyed_frame_cfilt() {
    static CASES: [(&str, Result<&str, DemangleError<'_>>); 14] = [
        (
            "_GLOBAL_$F$__7istreamiP9streambufP7ostream",
            Ok("istream::_GLOBAL_$F$(int, streambuf *, ostream *)"),
        ),
        (
            "_GLOBAL_$F$getline__7istreamPcic",
            Ok("istream::_GLOBAL_$F$getline(char *, int, char)"),
        ),
        (
            "_GLOBAL_$F$scan__7istreamPCce",
            Ok("istream::_GLOBAL_$F$scan(char const *,...)"),
        ),
        (
            "_GLOBAL_$F$vscan__9streambufPCcPcP3ios",
            Ok("streambuf::_GLOBAL_$F$vscan(char const *, char *, ios *)"),
        ),
        (
            "_GLOBAL_$F$cout",
            Err(DemangleError::InvalidNamespaceOnNamespacedGlobal("GLOBAL_")),
        ),
        (
            "_GLOBAL_$F$_un_link__9streambuf",
            Ok("streambuf::_GLOBAL_$F$_un_link(void)"),
        ),
        (
            "_GLOBAL_$F$init__7filebuf",
            Ok("filebuf::_GLOBAL_$F$init(void)"),
        ),
        (
            "_GLOBAL_$F$__as__22_IO_istream_withassignR7istream",
            Err(DemangleError::InvalidNamespaceOnNamespacedGlobal("GLOBAL_")),
        ),
        (
            "_GLOBAL_$F$_IO_stdin_",
            Err(DemangleError::InvalidNamespaceOnNamespacedGlobal("GLOBAL_")),
        ),
        (
            "_GLOBAL_$F$__8stdiobufP7__sFILE",
            Ok("stdiobuf::_GLOBAL_$F$(__sFILE *)"),
        ),
        (
            "_GLOBAL_$F$__default_terminate",
            Err(DemangleError::InvalidNamespaceOnNamespacedGlobal("GLOBAL_")),
        ),
        ("_GLOBAL_$F$terminate__Fv", Ok("_GLOBAL_$F$terminate(void)")),
        (
            "_GLOBAL_$F$_$_9type_info",
            Err(DemangleError::InvalidNamespaceOnNamespacedGlobal("GLOBAL_")),
        ),
        (
            "_GLOBAL_$F$before__C9type_infoRC9type_info",
            Ok("type_info::_GLOBAL_$F$before(type_info const &) const"),
        ),
    ];
    let mut config = DemangleConfig::new();
    config.demangle_global_keyed_frames = false;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), demangled.as_deref());
    }
}

#[test]
fn test_demangle_global_sym_keyed_frame_nocfilt() {
    static CASES: [(&str, &str); 14] = [
        (
            "_GLOBAL_$F$__7istreamiP9streambufP7ostream",
            "global frames keyed to istream::istream(int, streambuf *, ostream *)",
        ),
        (
            "_GLOBAL_$F$getline__7istreamPcic",
            "global frames keyed to istream::getline(char *, int, char)",
        ),
        (
            "_GLOBAL_$F$scan__7istreamPCce",
            "global frames keyed to istream::scan(char const *,...)",
        ),
        (
            "_GLOBAL_$F$vscan__9streambufPCcPcP3ios",
            "global frames keyed to streambuf::vscan(char const *, char *, ios *)",
        ),
        ("_GLOBAL_$F$cout", "global frames keyed to cout"),
        (
            "_GLOBAL_$F$_un_link__9streambuf",
            "global frames keyed to streambuf::_un_link(void)",
        ),
        (
            "_GLOBAL_$F$init__7filebuf",
            "global frames keyed to filebuf::init(void)",
        ),
        (
            "_GLOBAL_$F$__as__22_IO_istream_withassignR7istream",
            "global frames keyed to _IO_istream_withassign::operator=(istream &)",
        ),
        ("_GLOBAL_$F$_IO_stdin_", "global frames keyed to _IO_stdin_"),
        (
            "_GLOBAL_$F$__8stdiobufP7__sFILE",
            "global frames keyed to stdiobuf::stdiobuf(__sFILE *)",
        ),
        (
            "_GLOBAL_$F$__default_terminate",
            "global frames keyed to __default_terminate",
        ),
        (
            "_GLOBAL_$F$terminate__Fv",
            "global frames keyed to terminate(void)",
        ),
        (
            "_GLOBAL_$F$_$_9type_info",
            "global frames keyed to type_info::~type_info(void)",
        ),
        (
            "_GLOBAL_$F$before__C9type_infoRC9type_info",
            "global frames keyed to type_info::before(type_info const &) const",
        ),
    ];
    let mut config = DemangleConfig::new();
    config.demangle_global_keyed_frames = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_argument_array() {
    static CASES: [(&str, &str); 7] = [
        ("SetShadowAdjustments__15GeometryVehiclePA1_f", "GeometryVehicle::SetShadowAdjustments(float (*)[1])"),
        ("SetShadowAdjustments__7VehiclePA1_f", "Vehicle::SetShadowAdjustments(float (*)[1])"),
        ("simpler_array__FPA41_A24_Ci", "simpler_array(int const (*)[41][24])"),
        ("simpler_array__FPA41_A24_CUi", "simpler_array(unsigned int const (*)[41][24])"),
        ("an_arg_of_an_array_of_arrays_of_arrays__FPA38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_i", "an_arg_of_an_array_of_arrays_of_arrays(int (*)[38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38][38])"),
        ("an_arg_of_an_array_of_arrays_of_arrays__FPA41_A24_A38_A38_A38_A38_A38_A38_A38_A419_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A6_A0_i", "an_arg_of_an_array_of_arrays_of_arrays(int (*)[41][24][38][38][38][38][38][38][38][419][38][38][38][38][38][38][38][38][38][38][6][0])"),
        ("an_arg_of_an_array_of_arrays_of_arrays__FPA41_A24_A38_A38_A38_A38_A38_A38_A38_A419_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A6_A0_ifPA13_b", "an_arg_of_an_array_of_arrays_of_arrays(int (*)[41][24][38][38][38][38][38][38][38][419][38][38][38][38][38][38][38][38][38][38][6][0], float, bool (*)[13])"),
    ];
    let mut config = DemangleConfig::new();
    config.fix_array_length_arg = false;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_argument_array_fixed() {
    static CASES: [(&str, &str); 7] = [
        ("SetShadowAdjustments__15GeometryVehiclePA1_f", "GeometryVehicle::SetShadowAdjustments(float (*)[2])"),
        ("SetShadowAdjustments__7VehiclePA1_f", "Vehicle::SetShadowAdjustments(float (*)[2])"),
        ("simpler_array__FPA41_A24_Ci", "simpler_array(int const (*)[42][25])"),
        ("simpler_array__FPA41_A24_CUi", "simpler_array(unsigned int const (*)[42][25])"),
        ("an_arg_of_an_array_of_arrays_of_arrays__FPA38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_i", "an_arg_of_an_array_of_arrays_of_arrays(int (*)[39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39][39])"),
        ("an_arg_of_an_array_of_arrays_of_arrays__FPA41_A24_A38_A38_A38_A38_A38_A38_A38_A419_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A6_A0_i", "an_arg_of_an_array_of_arrays_of_arrays(int (*)[42][25][39][39][39][39][39][39][39][420][39][39][39][39][39][39][39][39][39][39][7][1])"),
        ("an_arg_of_an_array_of_arrays_of_arrays__FPA41_A24_A38_A38_A38_A38_A38_A38_A38_A419_A38_A38_A38_A38_A38_A38_A38_A38_A38_A38_A6_A0_ifPA13_b", "an_arg_of_an_array_of_arrays_of_arrays(int (*)[42][25][39][39][39][39][39][39][39][420][39][39][39][39][39][39][39][39][39][39][7][1], float, bool (*)[14])"),
    ];
    let mut config = DemangleConfig::new();
    config.fix_array_length_arg = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

// TODO: rename "template_with_return_type" to "templated_function" or smth
#[test]
fn test_demangle_template_with_return_type() {
    static CASES: [(&str, &str); 17] = [
        ("SetState__H1ZQ211CharacterAi4Loco_11CharacterAiPQ211CharacterAi12StateManager_v", "void CharacterAi::SetState<CharacterAi::Loco>(CharacterAi::StateManager *)"),
        ("SetState__H1ZQ35Other11CharacterAi4Loco_Q25Other11CharacterAiPQ35Other11CharacterAi12StateManager_v", "void Other::CharacterAi::SetState<Other::CharacterAi::Loco>(Other::CharacterAi::StateManager *)"),
        ("radBinarySearch__H1ZQ213radPs2CdDrive14DirectoryEntry_RCX01PCX01iPUi_b", "bool radBinarySearch<radPs2CdDrive::DirectoryEntry>(radPs2CdDrive::DirectoryEntry const &, radPs2CdDrive::DirectoryEntry const *, int, unsigned int *)"),

        ("DoThing__H2ZQ35Other11CharacterAi12StateManagerZQ35Other11CharacterAi4Loco_Q25Other11CharacterAiv_28some_return_with_underscores", "some_return_with_underscores Other::CharacterAi::DoThing<Other::CharacterAi::StateManager, Other::CharacterAi::Loco>(void)"),
        ("DoThing__H2ZQ35Other11CharacterAi12StateManagerZQ35Other11CharacterAi4Loco_Q25Other11CharacterAii_28some_return_with_underscores", "some_return_with_underscores Other::CharacterAi::DoThing<Other::CharacterAi::StateManager, Other::CharacterAi::Loco>(int)"),
        ("DoThing__H2ZQ35Other11CharacterAi12StateManagerZQ35Other11CharacterAi4Loco_Q25Other11CharacterAi_28some_return_with_underscores", "some_return_with_underscores Other::CharacterAi::DoThing<Other::CharacterAi::StateManager, Other::CharacterAi::Loco>()"),

        ("find__H2ZP5tNameZ5tName_X01X01RCX11G26random_access_iterator_tag_X01", "tName * find<tName *, tName>(tName *, tName *, tName const &, random_access_iterator_tag)"),
        ("BlendPriorities__H1ZQ218RadicalMathLibrary6Vector_6choreoPCQ26choreot13BlendPriority1ZX01iRX01_b", "bool choreo::BlendPriorities<RadicalMathLibrary::Vector>(choreo::BlendPriority<RadicalMathLibrary::Vector> const *, int, RadicalMathLibrary::Vector &)"),
        ("SetState__H9ZQ35Other11CharacterAi4LocoZQ35Other11CharacterAi12StateManagerZiZiZiZiZiZQ213radPs2CdDrive14DirectoryEntryZQ35Other11CharacterAi4Loco_Q25Other11CharacterAiRX11X01X21X31X41X51X61X71X81_v", "void Other::CharacterAi::SetState<Other::CharacterAi::Loco, Other::CharacterAi::StateManager, int, int, int, int, int, radPs2CdDrive::DirectoryEntry, Other::CharacterAi::Loco>(Other::CharacterAi::StateManager &, Other::CharacterAi::Loco, int, int, int, int, int, radPs2CdDrive::DirectoryEntry, Other::CharacterAi::Loco)"),
        ("_M_range_insert__H1ZPC5tName_Gt6vector2Z5tNameZt7s2alloc1Z5tNameP5tNameX01X01G20forward_iterator_tag_v", "void _M_range_insert<tName const *>(vector<tName, s2alloc<tName> >, tName *, tName const *, tName const *, forward_iterator_tag)"),
        ("_M_range_insert__H1ZPC5tName_t6vector2Z5tNameZt7s2alloc1Z5tNameP5tNameX00X00G20forward_iterator_tag_v", "void vector<tName, s2alloc<tName> >::_M_range_insert<tName const *>(tName *, tName const *, tName const *, forward_iterator_tag)"),
        ("_M_range_insert__H1ZPC5tName_GQ223some_allocation_libraryt6vector2Z5tNameZt7s2alloc1Z5tNameP5tNameX01X01G20forward_iterator_tag_v", "void _M_range_insert<tName const *>(some_allocation_library::vector<tName, s2alloc<tName> >, tName *, tName const *, tName const *, forward_iterator_tag)"),
        ("_M_range_insert__H1ZPC5tName_Q223some_allocation_libraryt6vector2Z5tNameZt7s2alloc1Z5tNameP5tNameX00X00G20forward_iterator_tag_v", "void some_allocation_library::vector<tName, s2alloc<tName> >::_M_range_insert<tName const *>(tName *, tName const *, tName const *, forward_iterator_tag)"),
        ("_M_range_insert__H1ZPC5tName_GQ223some_allocation_libraryt6vector2Z5tNameZt7s2alloc1Z5tNameP5tNameX01X01G20forward_iterator_tag_X01", "tName const * _M_range_insert<tName const *>(some_allocation_library::vector<tName, s2alloc<tName> >, tName *, tName const *, tName const *, forward_iterator_tag)"),
        // c++filt fails to demangle this symbol
        // ("SetState__H11ZQ35Other11CharacterAi4LocoZQ35Other11CharacterAi12StateManagerZiZiZiZiZiZiZiZQ213radPs2CdDrive14DirectoryEntryZQ35Other11CharacterAi4Loco_Q25Other11CharacterAiRX11X01X21X31X41X51X61X71X81X91X_10_1_v", ),

        ("indexof__H1Zf_PCX01T0_i", "int indexof<float>(float const *, float const *)"),
        ("indexof__H1Z11SGDMATERIAL_PCX01T0_i", "int indexof<SGDMATERIAL>(SGDMATERIAL const *, SGDMATERIAL const *)"),
        ("indexof__H1Z13SGDCOORDINATE_PCX01T0_i", "int indexof<SGDCOORDINATE>(SGDCOORDINATE const *, SGDCOORDINATE const *)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_avoid_duplicated_template_args_on_constr_destr() {
    static CASES: [(&str, &str); 5] = [
        ("__Q216radLoadInventoryt8SafeCast1Z22AnimCollisionEntityDSG", "radLoadInventory::SafeCast<AnimCollisionEntityDSG>::SafeCast(void)"),
        ("__Q216radLoadInventoryt8SafeCast1ZQ23sim13PhysicsObject", "radLoadInventory::SafeCast<sim::PhysicsObject>::SafeCast(void)"),
        ("__Q26choreot13BlendPriority1ZQ25poser9Transform", "choreo::BlendPriority<poser::Transform>::BlendPriority(void)"),
        ("_$_Q23simt5TList1ZPQ23sim15CollisionObject", "sim::TList<sim::CollisionObject *>::~TList(void)"),
        ("__Q23odst13pointer_templ1ZQ23ods6_groupRCQ23odst13pointer_templ1ZQ23ods6_group", "ods::pointer_templ<ods::_group>::pointer_templ(ods::pointer_templ<ods::_group> const &)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_more_templated_func_cases() {
    static CASES: [(&str, &str); 2] = [
        ("__push_heap__H3ZPt10MapElement2ZPQ23sim15CollisionObjectZP11DynaPhysDSGZiZt10MapElement2ZPQ23sim15CollisionObjectZP11DynaPhysDSG_X01X11X11X21_v", "void __push_heap<MapElement<sim::CollisionObject *, DynaPhysDSG *> *, int, MapElement<sim::CollisionObject *, DynaPhysDSG *> >(MapElement<sim::CollisionObject *, DynaPhysDSG *> *, int, int, MapElement<sim::CollisionObject *, DynaPhysDSG *>)"),
        ("__insertion_sort__H1ZPt10MapElement2ZPQ23sim15CollisionObjectZP11DynaPhysDSG_X01X01_v", "void __insertion_sort<MapElement<sim::CollisionObject *, DynaPhysDSG *> *>(MapElement<sim::CollisionObject *, DynaPhysDSG *> *, MapElement<sim::CollisionObject *, DynaPhysDSG *> *)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_operator_on_templated() {
    static CASES: [(&str, &str); 3] = [
        ("__as__t10MapElement2Z13tUidUnalignedZP5tPoseRCt10MapElement2Z13tUidUnalignedZP5tPose", "MapElement<tUidUnaligned, tPose *>::operator=(MapElement<tUidUnaligned, tPose *> const &)"),
        ("__as__t10MapElement2Z13tUidUnalignedZ13tUidUnalignedRCt10MapElement2Z13tUidUnalignedZ13tUidUnaligned", "MapElement<tUidUnaligned, tUidUnaligned>::operator=(MapElement<tUidUnaligned, tUidUnaligned> const &)"),
        ("__vc__t4List1Z15tSpriteParticles", "List<tSpriteParticle>::operator[](short)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_method_as_argument_() {
    // Code to generate first entry:
    // EE GCC 2.95.3 (SN BUILD v1.14)
    /*
    class SomeClass {
    public:
        void AClassMethod(void) {}
    };
    void class_method_args(void (SomeClass::*)(void)) {}
    */
    static CASES: [(&str, &str); 2] = [
        (
            "class_method_args__FPM9SomeClassFP9SomeClass_v",
            "class_method_args(void (SomeClass::*)())",
        ),
        (
            "class_method_args__FPM9SomeClassCFPC9SomeClass_v",
            "class_method_args(void (SomeClass::*)() const)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_method_as_argument_in_templated_single() {
    // EE GCC 2.95.3 (SN BUILD v1.14)
    /*
    namespace RadicalMathLibrary {
        class Vector {};
    }

    namespace choreo {
        class FootBlendDriver {
        public:
            void blendWithFoot(RadicalMathLibrary::Vector& vector) const {}
        };

        template <typename T>
        class BlendPriority {};

        template <typename T, typename U>
        void BlendDriverNoContext(
            U*,
            void (U::*)(T&) const,
            float,
            int,
            BlendPriority<T>*,
            int,
            int&
        ) {}
    }

    void triggerer(
        choreo::FootBlendDriver * a,
        float c,
        int d,
        choreo::BlendPriority<RadicalMathLibrary::Vector> * e,
        int f,
        int & g
    ) {
        choreo::BlendDriverNoContext(a, &choreo::FootBlendDriver::blendWithFoot, c, d, e, f, g);
    }
    */
    static CASES: [(&str, &str); 2] = [
        (
            "BlendDriverNoContext__H2ZQ218RadicalMathLibrary6VectorZQ26choreo15FootBlendDriver_6choreoPX11PMX11CFPCX11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverNoContext<RadicalMathLibrary::Vector, choreo::FootBlendDriver>(choreo::FootBlendDriver *, void (choreo::FootBlendDriver::*)(RadicalMathLibrary::Vector &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Vector> *, int, int &)",
        ),
        (
            // Non const method variant.
            "BlendDriverNoContext__H2ZQ218RadicalMathLibrary6VectorZQ26choreo15FootBlendDriver_6choreoPX11PMX11FPX11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverNoContext<RadicalMathLibrary::Vector, choreo::FootBlendDriver>(choreo::FootBlendDriver *, void (choreo::FootBlendDriver::*)(RadicalMathLibrary::Vector &), float, int, choreo::BlendPriority<RadicalMathLibrary::Vector> *, int, int &)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_method_as_argument_in_templated_many() {
    static CASES: [(&str, &str); 8] = [
        (
            "BlendDriverNoContext__H2ZQ218RadicalMathLibrary6VectorZQ26choreo15FootBlendDriver_6choreoPX11PMX11CFPCX11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverNoContext<RadicalMathLibrary::Vector, choreo::FootBlendDriver>(choreo::FootBlendDriver *, void (choreo::FootBlendDriver::*)(RadicalMathLibrary::Vector &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Vector> *, int, int &)",
        ),
        (
            "BlendDriverNoContext__H2ZQ218RadicalMathLibrary10QuaternionZQ26choreo15FootBlendDriver_6choreoPX11PMX11CFPCX11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverNoContext<RadicalMathLibrary::Quaternion, choreo::FootBlendDriver>(choreo::FootBlendDriver *, void (choreo::FootBlendDriver::*)(RadicalMathLibrary::Quaternion &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Quaternion> *, int, int &)",
        ),
        (
            "BlendDriverNoContext__H2ZfZQ26choreo15FootBlendDriver_6choreoPX11PMX11CFPCX11_X01fiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverNoContext<float, choreo::FootBlendDriver>(choreo::FootBlendDriver *, float (choreo::FootBlendDriver::*)() const, float, int, choreo::BlendPriority<float> *, int, int &)",
        ),
        (
            "BlendDriverWithContext__H3ZQ218RadicalMathLibrary6VectorZiZQ26choreo16JointBlendDriver_6choreoX11PX21PMX21CFPCX21X11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverWithContext<RadicalMathLibrary::Vector, int, choreo::JointBlendDriver>(int, choreo::JointBlendDriver *, void (choreo::JointBlendDriver::*)(int, RadicalMathLibrary::Vector &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Vector> *, int, int &)",
        ),
        (
            "BlendDriverWithContext__H3ZQ218RadicalMathLibrary10QuaternionZiZQ26choreo16JointBlendDriver_6choreoX11PX21PMX21CFPCX21X11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverWithContext<RadicalMathLibrary::Quaternion, int, choreo::JointBlendDriver>(int, choreo::JointBlendDriver *, void (choreo::JointBlendDriver::*)(int, RadicalMathLibrary::Quaternion &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Quaternion> *, int, int &)",
        ),
        (
            "BlendDriverWithContext__H3ZQ218RadicalMathLibrary6VectorZRCQ25poser9TransformZQ26choreo15RootBlendDriver_6choreoX11PX21PMX21CFPCX21X11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverWithContext<RadicalMathLibrary::Vector, poser::Transform const &, choreo::RootBlendDriver>(poser::Transform const &, choreo::RootBlendDriver *, void (choreo::RootBlendDriver::*)(poser::Transform const &, RadicalMathLibrary::Vector &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Vector> *, int, int &)",
        ),
        (
            "BlendDriverWithContext__H3ZQ218RadicalMathLibrary10QuaternionZRCQ25poser9TransformZQ26choreo15RootBlendDriver_6choreoX11PX21PMX21CFPCX21X11RX01_vfiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverWithContext<RadicalMathLibrary::Quaternion, poser::Transform const &, choreo::RootBlendDriver>(poser::Transform const &, choreo::RootBlendDriver *, void (choreo::RootBlendDriver::*)(poser::Transform const &, RadicalMathLibrary::Quaternion &) const, float, int, choreo::BlendPriority<RadicalMathLibrary::Quaternion> *, int, int &)",
        ),
        (
            "BlendDriverNoContext__H2ZfZQ26choreo15RootBlendDriver_6choreoPX11PMX11CFPCX11_X01fiPQ26choreot13BlendPriority1ZX01iRi_v",
            "void choreo::BlendDriverNoContext<float, choreo::RootBlendDriver>(choreo::RootBlendDriver *, float (choreo::RootBlendDriver::*)() const, float, int, choreo::BlendPriority<float> *, int, int &)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_same_sym_but_different_mangling() {
    // Different g++ versions may mangle the symbol differently, but following
    // the same "mangling ABI", meaning they are "equivalent".
    static CASES: [(&str, &str); 2] = [
        // EE GCC 2.9 build 990721
        (
            "Debug_Assert__FPcT0T0i",
            "Debug_Assert(char *, char *, char *, int)",
        ),
        // EE GCC 2.96 build 001003-1
        (
            "Debug_Assert__FPcN20i",
            "Debug_Assert(char *, char *, char *, int)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_128bits_integers_cfilt() {
    static CASES: [(&str, &str); 2] = [
        (
            "Tim2LoadTexture__FiUiiiiPUI80",
            "Tim2LoadTexture(int, unsigned int, int, int, int, unsigned int128_t *)",
        ),
        ("signed_128__FRCI80", "signed_128(int128_t const &)"),
    ];
    let mut config = DemangleConfig::new();
    config.fix_extension_int = false;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_128bits_integers_fix() {
    static CASES: [(&str, &str); 2] = [
        (
            "Tim2LoadTexture__FiUiiiiPUI80",
            "Tim2LoadTexture(int, unsigned int, int, int, int, __uint128_t *)",
        ),
        ("signed_128__FRCI80", "signed_128(__int128_t const &)"),
    ];
    let mut config = DemangleConfig::new();
    config.fix_extension_int = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_template_with_enum_value() {
    // First entry can be generated with
    /*
    enum G3DTRANSFORMSTATETYPE {
        Number0
    };
    template <G3DTRANSFORMSTATETYPE N>
    class CAutoTransform {
    public:
        void Pop() {}
    };
    void test(CAutoTransform<Number0> & a) {
        a.Pop();
    }
    */
    static CASES: [(&str, &str); 5] = [
        (
            "Pop__t14CAutoTransform121G3DTRANSFORMSTATETYPE0",
            "CAutoTransform<0>::Pop(void)",
        ),
        (
            "__tft14CAutoTransform121G3DTRANSFORMSTATETYPE0",
            "CAutoTransform<0> type_info function",
        ),
        (
            "__tit14CAutoTransform121G3DTRANSFORMSTATETYPE0",
            "CAutoTransform<0> type_info node",
        ),
        (
            "_$_t14CAutoTransform121G3DTRANSFORMSTATETYPE0",
            "CAutoTransform<0>::~CAutoTransform(void)",
        ),
        (
            "_vt$t14CAutoTransform121G3DTRANSFORMSTATETYPE0",
            "CAutoTransform<0> virtual table",
        ),
    ];
    let mut config = DemangleConfig::new();
    config.fix_extension_int = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_templated_function_with_value_reuse() {
    static CASES: [(&str, &str); 2] = [
        /*
        template <typename T, unsigned int N>
        class fixed_array {};

        class _LIGHTCOMPAREDATA {};

        template <  int N>
        void _SortLightCompareData(fixed_array<_LIGHTCOMPAREDATA, N> &, float, int) {}

        void test(fixed_array<_LIGHTCOMPAREDATA, 4> & a , float b, int c) {
            _SortLightCompareData<4>(a, b, c);
        }
        */
        (
            "_SortLightCompareData__H1i4_Rt11fixed_array2Z17_LIGHTCOMPAREDATAUiY01fi_v",
            "void _SortLightCompareData<4>(fixed_array<_LIGHTCOMPAREDATA, 4> &, float, int)",
        ),
        (
            "_SortLightCompareData__H1im4_Rt11fixed_array2Z17_LIGHTCOMPAREDATAiY01fi_v",
            "void _SortLightCompareData<-4>(fixed_array<_LIGHTCOMPAREDATA, -4> &, float, int)",
        ),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_array_without_pointer_cfilt() {
    static CASES: [(&str, &str); 9] = [
        // TODO: add a flag to allow emitting the cursed valid syntax instead
        // of the invalid one.
        // TODO: figure out how to get g++ to emit type_info stuff for plain arrays.
        // why on earth is this valid syntax?
        /*
        template <typename T>
        T * _fixed_array_verifyrange(unsigned int, unsigned int) {}

        int (*test2(unsigned int a, unsigned int b))[4] {
            return _fixed_array_verifyrange<int [4]>(a, b);
        }
        */
        (
            "_fixed_array_verifyrange__H1ZA3_i_UiUi_PX01",
            "int [3] * _fixed_array_verifyrange<int [3]>(unsigned int, unsigned int)",
        ),
        ("__tiA3_i", "int [3] type_info node"),
        ("__tiA3_f", "float [3] type_info node"),
        ("__tiA3_A3_f", "float [3][3] type_info node"),
        ("__tfA3_f", "float [3] type_info function"),
        ("__tfA3_A3_f", "float [3][3] type_info function"),
        ("__tfA3_i", "int [3] type_info function"),
        (
            "_fixed_array_verifyrange__H1ZA3_A3_f_UiUi_PX01",
            "float [3][3] * _fixed_array_verifyrange<float [3][3]>(unsigned int, unsigned int)",
        ),
        (
            "_fixed_array_verifyrange__H1ZA3_f_UiUi_PX01",
            "float [3] * _fixed_array_verifyrange<float [3]>(unsigned int, unsigned int)",
        ),
    ];
    let mut config = DemangleConfig::new();
    config.fix_array_length_arg = false;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_array_without_pointer_fixed() {
    static CASES: [(&str, &str); 9] = [
        // TODO: add a flag to allow emitting the cursed valid syntax instead
        // of the invalid one.
        // why on earth is this valid syntax?
        /*
        template <typename T>
        T * _fixed_array_verifyrange(unsigned int, unsigned int) {}

        int (*test2(unsigned int a, unsigned int b))[4] {
            return _fixed_array_verifyrange<int [4]>(a, b);
        }
        */
        (
            "_fixed_array_verifyrange__H1ZA3_i_UiUi_PX01",
            "int [4] * _fixed_array_verifyrange<int [4]>(unsigned int, unsigned int)",
        ),
        ("__tiA3_i", "int [4] type_info node"),
        ("__tiA3_f", "float [4] type_info node"),
        ("__tiA3_A3_f", "float [4][4] type_info node"),
        ("__tfA3_f", "float [4] type_info function"),
        ("__tfA3_A3_f", "float [4][4] type_info function"),
        ("__tfA3_i", "int [4] type_info function"),
        (
            "_fixed_array_verifyrange__H1ZA3_A3_f_UiUi_PX01",
            "float [4][4] * _fixed_array_verifyrange<float [4][4]>(unsigned int, unsigned int)",
        ),
        (
            "_fixed_array_verifyrange__H1ZA3_f_UiUi_PX01",
            "float [4] * _fixed_array_verifyrange<float [4]>(unsigned int, unsigned int)",
        ),
    ];
    let mut config = DemangleConfig::new();
    config.fix_array_length_arg = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_function_pointer_returning_pointer_to_array_cfilt() {
    static CASES: [(&str, &str); 1] = [
        // This can be also written like this.
        // Hopefully this is more simple to understand to the reader.
        /*
        typedef int  (*Arr)[4];
        typedef Arr (* First)(void);
        typedef void (* Second)(Arr);
        void InitDrawEnv(First, First, Second, Second) {}
        */
        ("InitDrawEnv__FPFv_PA3_iT0PFPA3_i_vT2", "InitDrawEnv(int (*(*)(void))[3], int (*(*)(void))[3], void (*)(int (*)[3]), void (*)(int (*)[3]))"),
    ];
    let mut config = DemangleConfig::new();
    config.fix_array_length_arg = false;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

#[test]
fn test_demangle_function_pointer_returning_pointer_to_array_fixed() {
    static CASES: [(&str, &str); 1] = [
        // This can be also written like this.
        // Hopefully this is more simple to understand to the reader.
        /*
        typedef int  (*Arr)[4];
        typedef Arr (* First)(void);
        typedef void (* Second)(Arr);
        void InitDrawEnv(First, First, Second, Second) {}
        */
        ("InitDrawEnv__FPFv_PA3_iT0PFPA3_i_vT2", "InitDrawEnv(int (*(*)(void))[4], int (*(*)(void))[4], void (*)(int (*)[4]), void (*)(int (*)[4]))"),
    ];
    let mut config = DemangleConfig::new();
    config.fix_array_length_arg = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

/*
class SomethingSilly {
public:
    template <typename T>
    int (*an_array(T a) const) [3] {}
};
*/
/*
(
    "an_array__H1Zi_C14SomethingSillyX01_PA3_i",
    "int (*)[3] SomethingSilly::an_array<int>(int) const",
),
*/

/*
#[test]
fn test_demangle_single() {
    static CASES: [(&str, &str); 1] = [
        ("actual_function__FRt10SomeVector2Z4NodeR13TestAllocator17AllocatorInstanceG4Node", "actual_function(SomeVector<Node, AllocatorInstance> &, Node)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}
*/

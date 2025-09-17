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
    static CASES: [(&str, &str); 8] = [
        ("AddPair__Q33sim16CollisionManager4AreaPQ23sim15CollisionObjectT1", "sim::CollisionManager::Area::AddPair(sim::CollisionObject *, sim::CollisionObject *)"),
        ("CollisionEvent__Q23sim20CollisionSolverAgentPQ23sim8SimStateiT1iRCQ218RadicalMathLibrary6VectorffPPQ23sim15SimulatedObjectT8", "sim::CollisionSolverAgent::CollisionEvent(sim::SimState *, int, sim::SimState *, int, RadicalMathLibrary::Vector const &, float, float, sim::SimulatedObject **, sim::SimulatedObject **)"),
        ("EdgeEdge__Q23sim20SubCollisionDetectorRbRQ218RadicalMathLibrary6VectorT2fT2T2fT2ffPQ23sim15CollisionVolumeT11_", "sim::SubCollisionDetector::EdgeEdge(bool &, RadicalMathLibrary::Vector &, RadicalMathLibrary::Vector &, float, RadicalMathLibrary::Vector &, RadicalMathLibrary::Vector &, float, RadicalMathLibrary::Vector &, float, float, sim::CollisionVolume *, sim::CollisionVolume *)"),
        ("AddPair__Q33sim16CollisionManager4AreaPQ23sim15CollisionObjectT0", "sim::CollisionManager::Area::AddPair(sim::CollisionObject *, sim::CollisionManager::Area)"),
        ("AddPair__FQ33sim16CollisionManager4AreaPQ23sim15CollisionObjectT0", "AddPair(sim::CollisionManager::Area, sim::CollisionObject *, sim::CollisionManager::Area)"),
        ("do_thing__C6StupidG6StupidT1", "Stupid::do_thing(Stupid, Stupid) const"),
        ("do_thing__C6StupidRC6StupidT1", "Stupid::do_thing(Stupid const &, Stupid const &) const"),
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
    static CASES: [(&str, &str); 6] = [
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
    static CASES: [(&str, &str); 3] = [
        ("Printf__7ConsolePce", "Console::Printf(char *,...)"),
        (
            "StrPrintf__6choreoPciPCce",
            "choreo::StrPrintf(char *, int, char const *,...)",
        ),
        ("printf__3p3dPCce", "p3d::printf(char const *,...)"),
    ];
    let config = DemangleConfig::new();

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
    static CASES: [(&str, &str); 10] = [
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
    static CASES: [(&str, &str); 5] = [
        ("set_terminate__FPFv_v", "set_terminate(void (*)(void))"),
        (
            "set_unexpected__FPFv_v",
            "set_unexpected(void (*)(void))",
        ),
        ("pointerness__FPFiGt9Something1x42_t9Something1x39iPFiRt9Something1x42_RCt9Something1x39", "pointerness(Something<39> (*)(int, Something<42>), int, Something<39> const &(*)(int, Something<42> &))"),
        ("pointerness__FPFiGt9Something1x42_t9Something1x39iPFiRt9Something1x42_RCt9Something1x39RFPCce_RQ55First6Second5Third6Fourth1A", "pointerness(Something<39> (*)(int, Something<42>), int, Something<39> const &(*)(int, Something<42> &), First::Second::Third::Fourth::A &(&)(char const *,...))"),
        ("InstallShader__14pddiBaseShaderPCcPFP17pddiRenderContextPCcPCc_P14pddiBaseShaderT1", "pddiBaseShader::InstallShader(char const *, pddiBaseShader *(*)(pddiRenderContext *, char const *, char const *), char const *)"),
    ];
    let config = DemangleConfig::new();

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &config).as_deref(), Ok(demangled));
    }
}

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

// TODO: does templated global objects exist?

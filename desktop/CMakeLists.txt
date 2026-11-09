find_package(Qt6 6.5 REQUIRED COMPONENTS Widgets LinguistTools)
qt_standard_project_setup()

set(OPENRULES_RUNTIME_EXECUTABLE "" CACHE FILEPATH "Built Rust openrules-runtime to stage beside the dashboard")
option(OPENRULES_DEPLOY_QT "Stage Qt runtime dependencies on install" OFF)
configure_file(src/build_info.h.in "${CMAKE_CURRENT_BINARY_DIR}/build_info.h" @ONLY)

add_library(openrules-widgets STATIC
    src/runtime_client.cpp src/runtime_client.h
    src/main_window.cpp src/main_window.h
    src/settings.cpp src/settings_panels.cpp src/settings_panels.h
    src/governance_panel.cpp src/governance_panel.h src/governance_editor.h
    src/theme.cpp src/theme.h
    src/live_text.cpp src/live_text.h
    src/notification_center.cpp src/notification_center.h
    src/repositories.cpp src/conversations.cpp src/automations.cpp
    src/repository_picker.h src/compact_text_edit.h src/full_row_table.h
    resources/openrules_ui.qrc
)
set_target_properties(openrules-widgets PROPERTIES AUTORCC ON)
target_compile_features(openrules-widgets PUBLIC cxx_std_20)
target_include_directories(openrules-widgets PUBLIC src "${CMAKE_CURRENT_BINARY_DIR}")
target_link_libraries(openrules-widgets PUBLIC Qt6::Widgets)
if(MSVC)
    target_compile_options(openrules-widgets PUBLIC /utf-8)
endif()

qt_add_executable(openrules-desktop WIN32 src/main.cpp)
target_link_libraries(openrules-desktop PRIVATE openrules-widgets)
qt_add_translations(openrules-desktop
    TS_FILES translations/openrules_vi.ts translations/openrules_zh_CN.ts
    RESOURCE_PREFIX "/i18n")
install(TARGETS openrules-desktop RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")

if(OPENRULES_RUNTIME_EXECUTABLE)
    if(NOT IS_ABSOLUTE "${OPENRULES_RUNTIME_EXECUTABLE}" OR NOT EXISTS "${OPENRULES_RUNTIME_EXECUTABLE}")
        message(FATAL_ERROR "OPENRULES_RUNTIME_EXECUTABLE must be the absolute path to a built runtime")
    endif()
    if(WIN32)
        set(runtime_name openrules-runtime.exe)
    else()
        set(runtime_name openrules-runtime)
    endif()
    add_custom_target(openrules-stage-runtime ALL
        COMMAND "${CMAKE_COMMAND}"
            "-DSOURCE=${OPENRULES_RUNTIME_EXECUTABLE}"
            "-DDEST=$<TARGET_FILE_DIR:openrules-desktop>/${runtime_name}"
            -P "${CMAKE_CURRENT_SOURCE_DIR}/stage_runtime.cmake"
        DEPENDS openrules-desktop "${OPENRULES_RUNTIME_EXECUTABLE}"
        VERBATIM
    )
    install(PROGRAMS "$<TARGET_FILE_DIR:openrules-desktop>/${runtime_name}"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES "$<TARGET_FILE_DIR:openrules-desktop>/${runtime_name}.sha256"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
endif()

if(OPENRULES_DEPLOY_QT)
    qt_generate_deploy_app_script(TARGET openrules-desktop OUTPUT_SCRIPT deploy_script
        NO_UNSUPPORTED_PLATFORM_ERROR)
    install(SCRIPT "${deploy_script}")
endif()

if(BUILD_TESTING)
    find_package(Qt6 REQUIRED COMPONENTS Test)
    add_executable(openrules-qt-tests tests/desktop_test.cpp tests/repository_cases.h)
    target_link_libraries(openrules-qt-tests PRIVATE openrules-widgets Qt6::Test)
    add_dependencies(openrules-qt-tests openrules-desktop)
    qt_add_resources(openrules-qt-tests "test_i18n" PREFIX "/i18n" BASE "${CMAKE_CURRENT_BINARY_DIR}"
        FILES "${CMAKE_CURRENT_BINARY_DIR}/openrules_vi.qm" "${CMAKE_CURRENT_BINARY_DIR}/openrules_zh_CN.qm")
    add_test(NAME openrules.desktop COMMAND openrules-qt-tests)
    set_tests_properties(openrules.desktop PROPERTIES
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen;OPENRULES_TEST_QM=${CMAKE_CURRENT_BINARY_DIR}/openrules_vi.qm;OPENRULES_TEST_ZH_QM=${CMAKE_CURRENT_BINARY_DIR}/openrules_zh_CN.qm"
        TIMEOUT 120)
    if(WIN32)
        set_tests_properties(openrules.desktop PROPERTIES
            ENVIRONMENT_MODIFICATION "PATH=path_list_prepend:$<TARGET_FILE_DIR:Qt6::Core>")
    endif()
    if(OPENRULES_RUNTIME_EXECUTABLE)
        set_property(TEST openrules.desktop APPEND PROPERTY
            ENVIRONMENT "OPENRULES_TEST_RUNTIME=${OPENRULES_RUNTIME_EXECUTABLE}")
    endif()
endif()

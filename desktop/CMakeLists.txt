set(runtime_target "${identity_product_id}-runtime")
set(desktop_target "${identity_product_id}-desktop")
set(widgets_target "${identity_product_id}-widgets")
set(tests_target "${identity_product_id}-qt-tests")
set(stage_target "${identity_product_id}-stage-runtime")
set(test_name "${identity_product_id}.desktop")
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/resources/${identity_product_id}_ui.qrc" resource_definition)
string(FIND "${resource_definition}" "prefix=\"/${identity_product_id}\"" resource_prefix)
if(resource_prefix EQUAL -1)
    message(FATAL_ERROR "Qt resource prefix must match application identity")
endif()

find_package(Qt6 6.5 REQUIRED COMPONENTS Widgets LinguistTools)
qt_standard_project_setup()

set(CIRNETRULES_RUNTIME_EXECUTABLE "" CACHE FILEPATH "Built Rust ${runtime_target} to stage beside the dashboard")
option(CIRNETRULES_DEPLOY_QT "Stage Qt runtime dependencies on install" OFF)
configure_file(src/build_info.h.in "${CMAKE_CURRENT_BINARY_DIR}/build_info.h" @ONLY)

add_library(${widgets_target} STATIC
    src/runtime_client.cpp src/runtime_client.h
    src/main_window.cpp src/main_window.h
    src/settings.cpp src/settings_panels.cpp src/settings_panels.h
    src/governance_panel.cpp src/governance_panel.h src/governance_editor.h
    src/theme.cpp src/theme.h
    src/live_text.cpp src/live_text.h
    src/notification_center.cpp src/notification_center.h
    src/repositories.cpp src/conversations.cpp src/automations.cpp
    src/repository_picker.h src/compact_text_edit.h src/full_row_table.h
    resources/${identity_product_id}_ui.qrc
)
set_target_properties(${widgets_target} PROPERTIES AUTORCC ON)
target_compile_features(${widgets_target} PUBLIC cxx_std_20)
target_include_directories(${widgets_target} PUBLIC src "${CMAKE_CURRENT_BINARY_DIR}")
target_link_libraries(${widgets_target} PUBLIC Qt6::Widgets)
if(MSVC)
    target_compile_options(${widgets_target} PUBLIC /utf-8)
endif()

qt_add_executable(${desktop_target} WIN32 src/main.cpp)
target_link_libraries(${desktop_target} PRIVATE ${widgets_target})
qt_add_translations(${desktop_target}
    TS_FILES translations/${identity_product_id}_vi.ts translations/${identity_product_id}_zh_CN.ts
    RESOURCE_PREFIX "/i18n")
install(TARGETS ${desktop_target} RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")

if(CIRNETRULES_RUNTIME_EXECUTABLE)
    if(NOT IS_ABSOLUTE "${CIRNETRULES_RUNTIME_EXECUTABLE}" OR NOT EXISTS "${CIRNETRULES_RUNTIME_EXECUTABLE}")
        message(FATAL_ERROR "CIRNETRULES_RUNTIME_EXECUTABLE must be the absolute path to a built runtime")
    endif()
    if(WIN32)
        set(runtime_name ${runtime_target}.exe)
    else()
        set(runtime_name ${runtime_target})
    endif()
    add_custom_target(${stage_target} ALL
        COMMAND "${CMAKE_COMMAND}"
            "-DSOURCE=${CIRNETRULES_RUNTIME_EXECUTABLE}"
            "-DDEST=$<TARGET_FILE_DIR:${desktop_target}>/${runtime_name}"
            -P "${CMAKE_CURRENT_SOURCE_DIR}/stage_runtime.cmake"
        DEPENDS ${desktop_target} "${CIRNETRULES_RUNTIME_EXECUTABLE}"
        VERBATIM
    )
    install(PROGRAMS "$<TARGET_FILE_DIR:${desktop_target}>/${runtime_name}"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES "$<TARGET_FILE_DIR:${desktop_target}>/${runtime_name}.sha256"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
endif()

if(CIRNETRULES_DEPLOY_QT)
    qt_generate_deploy_app_script(TARGET ${desktop_target} OUTPUT_SCRIPT deploy_script
        NO_UNSUPPORTED_PLATFORM_ERROR)
    install(SCRIPT "${deploy_script}")
endif()

if(BUILD_TESTING)
    find_package(Qt6 REQUIRED COMPONENTS Test)
    add_executable(${tests_target} tests/desktop_test.cpp tests/repository_cases.h)
    target_link_libraries(${tests_target} PRIVATE ${widgets_target} Qt6::Test)
    add_dependencies(${tests_target} ${desktop_target})
    qt_add_resources(${tests_target} "test_i18n" PREFIX "/i18n" BASE "${CMAKE_CURRENT_BINARY_DIR}"
        FILES "${CMAKE_CURRENT_BINARY_DIR}/${identity_product_id}_vi.qm" "${CMAKE_CURRENT_BINARY_DIR}/${identity_product_id}_zh_CN.qm")
    add_test(NAME ${test_name} COMMAND ${tests_target})
    set_tests_properties(${test_name} PROPERTIES
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen;CIRNETRULES_TEST_QM=${CMAKE_CURRENT_BINARY_DIR}/${identity_product_id}_vi.qm;CIRNETRULES_TEST_ZH_QM=${CMAKE_CURRENT_BINARY_DIR}/${identity_product_id}_zh_CN.qm"
        TIMEOUT 120)
    if(WIN32)
        set_tests_properties(${test_name} PROPERTIES
            ENVIRONMENT_MODIFICATION "PATH=path_list_prepend:$<TARGET_FILE_DIR:Qt6::Core>")
    endif()
    if(CIRNETRULES_RUNTIME_EXECUTABLE)
        set_property(TEST ${test_name} APPEND PROPERTY
            ENVIRONMENT "CIRNETRULES_TEST_RUNTIME=${CIRNETRULES_RUNTIME_EXECUTABLE}")
    endif()
endif()

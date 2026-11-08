find_package(Qt6 6.5 REQUIRED COMPONENTS Widgets LinguistTools)
qt_standard_project_setup()

set(CGACP_RUNTIME_EXECUTABLE "" CACHE FILEPATH "Built Rust cgacp-runtime to stage beside the dashboard")
option(CGACP_DEPLOY_QT "Stage Qt runtime dependencies on install" OFF)
configure_file(src/build_info.h.in "${CMAKE_CURRENT_BINARY_DIR}/build_info.h" @ONLY)

add_library(cgacp-widgets STATIC
    src/runtime_client.cpp src/runtime_client.h
    src/main_window.cpp src/main_window.h
    src/repositories.cpp src/conversations.cpp src/automations.cpp
    resources/cgacp_ui.qrc
)
set_target_properties(cgacp-widgets PROPERTIES AUTORCC ON)
target_compile_features(cgacp-widgets PUBLIC cxx_std_20)
target_include_directories(cgacp-widgets PUBLIC src "${CMAKE_CURRENT_BINARY_DIR}")
target_link_libraries(cgacp-widgets PUBLIC Qt6::Widgets)
if(MSVC)
    target_compile_options(cgacp-widgets PUBLIC /utf-8)
endif()

qt_add_executable(cgacp-dashboard WIN32 src/main.cpp)
target_link_libraries(cgacp-dashboard PRIVATE cgacp-widgets)
qt_add_translations(cgacp-dashboard TS_FILES translations/cgacp_vi.ts RESOURCE_PREFIX "/i18n")
install(TARGETS cgacp-dashboard RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")

if(CGACP_RUNTIME_EXECUTABLE)
    if(NOT IS_ABSOLUTE "${CGACP_RUNTIME_EXECUTABLE}" OR NOT EXISTS "${CGACP_RUNTIME_EXECUTABLE}")
        message(FATAL_ERROR "CGACP_RUNTIME_EXECUTABLE must be the absolute path to a built runtime")
    endif()
    if(WIN32)
        set(runtime_name cgacp-runtime.exe)
    else()
        set(runtime_name cgacp-runtime)
    endif()
    add_custom_target(cgacp-stage-runtime ALL
        COMMAND "${CMAKE_COMMAND}"
            "-DSOURCE=${CGACP_RUNTIME_EXECUTABLE}"
            "-DDEST=$<TARGET_FILE_DIR:cgacp-dashboard>/${runtime_name}"
            -P "${CMAKE_CURRENT_SOURCE_DIR}/stage_runtime.cmake"
        DEPENDS cgacp-dashboard "${CGACP_RUNTIME_EXECUTABLE}"
        VERBATIM
    )
    install(PROGRAMS "$<TARGET_FILE_DIR:cgacp-dashboard>/${runtime_name}"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES "$<TARGET_FILE_DIR:cgacp-dashboard>/${runtime_name}.sha256"
        DESTINATION "${CMAKE_INSTALL_BINDIR}")
endif()

if(CGACP_DEPLOY_QT)
    qt_generate_deploy_app_script(TARGET cgacp-dashboard OUTPUT_SCRIPT deploy_script
        NO_UNSUPPORTED_PLATFORM_ERROR)
    install(SCRIPT "${deploy_script}")
endif()

if(BUILD_TESTING)
    find_package(Qt6 REQUIRED COMPONENTS Test)
    add_executable(cgacp-qt-tests tests/desktop_test.cpp)
    target_link_libraries(cgacp-qt-tests PRIVATE cgacp-widgets Qt6::Test)
    add_dependencies(cgacp-qt-tests cgacp-dashboard)
    add_test(NAME cgacp.desktop COMMAND cgacp-qt-tests)
    set_tests_properties(cgacp.desktop PROPERTIES
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen;CGACP_TEST_QM=${CMAKE_CURRENT_BINARY_DIR}/cgacp_vi.qm"
        TIMEOUT 35)
    if(WIN32)
        set_tests_properties(cgacp.desktop PROPERTIES
            ENVIRONMENT_MODIFICATION "PATH=path_list_prepend:$<TARGET_FILE_DIR:Qt6::Core>")
    endif()
    if(CGACP_RUNTIME_EXECUTABLE)
        set_property(TEST cgacp.desktop APPEND PROPERTY
            ENVIRONMENT "CGACP_TEST_RUNTIME=${CGACP_RUNTIME_EXECUTABLE}")
    endif()
endif()

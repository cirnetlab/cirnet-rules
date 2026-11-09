cmake_minimum_required(VERSION 3.24)
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/VERSION" OPENRULES_VERSION LIMIT_COUNT 1)
if(NOT OPENRULES_VERSION MATCHES "^[0-9]+\\.[0-9]+\\.[0-9]+$")
    message(FATAL_ERROR "VERSION must contain a numeric application version")
endif()
project(openrules VERSION "${OPENRULES_VERSION}" LANGUAGES CXX)

include(CTest)
include(GNUInstallDirs)
add_subdirectory(desktop)

install(FILES LICENSE THIRD_PARTY_NOTICES.md DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/openrules")
foreach(language IN ITEMS en vi)
    install(FILES
        "${language}/references/dashboard.md"
        "${language}/references/dashboard-architecture.md"
        "${language}/references/dashboard-release.md"
        "${language}/references/governance-workspace.md"
        "${language}/references/adapter-capabilities.md"
        "${language}/references/openrules-identity.md"
        "${language}/CHANGELOG.md"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/openrules/${language}")
endforeach()

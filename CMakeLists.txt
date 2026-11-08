cmake_minimum_required(VERSION 3.24)
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/VERSION" CGACP_VERSION LIMIT_COUNT 1)
if(NOT CGACP_VERSION MATCHES "^[0-9]+\\.[0-9]+\\.[0-9]+$")
    message(FATAL_ERROR "VERSION must contain a numeric application version")
endif()
project(cirnet-rules VERSION "${CGACP_VERSION}" LANGUAGES CXX)

include(CTest)
include(GNUInstallDirs)
add_subdirectory(desktop)

install(FILES LICENSE THIRD_PARTY_NOTICES.md DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/cgacp")
foreach(language IN ITEMS en vi)
    install(FILES
        "${language}/references/dashboard.md"
        "${language}/references/dashboard-architecture.md"
        "${language}/references/dashboard-release.md"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/doc/cgacp/${language}")
endforeach()

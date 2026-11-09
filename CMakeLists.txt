cmake_minimum_required(VERSION 3.24)
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/VERSION" CIRNETRULES_VERSION LIMIT_COUNT 1)
if(NOT CIRNETRULES_VERSION MATCHES "^[0-9]+\\.[0-9]+\\.[0-9]+$")
    message(FATAL_ERROR "VERSION must contain a numeric application version")
endif()
include(scripts/application_identity.cmake)
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${identity_manifest}" VERSION Cargo.toml)
file(STRINGS Cargo.toml cargo_identity REGEX "^(name|version) = ")
list(GET cargo_identity 0 cargo_package)
list(GET cargo_identity 1 cargo_version)
if(NOT cargo_package STREQUAL "name = \"${identity_product_id}\"" OR
   NOT cargo_version STREQUAL "version = \"${CIRNETRULES_VERSION}\"")
    message(FATAL_ERROR "Cargo package identity and VERSION must match application metadata")
endif()
project("${identity_product_id}" VERSION "${CIRNETRULES_VERSION}" LANGUAGES CXX)

include(CTest)
include(GNUInstallDirs)
add_subdirectory(desktop)

set(documentation_dir "${CMAKE_INSTALL_DATADIR}/doc/${identity_product_id}")
install(FILES LICENSE THIRD_PARTY_NOTICES.md application-identity.json identity-legacy-allowlist.json
    DESTINATION "${documentation_dir}")
install(FILES en/schemas/dashboard-ipc.schema.json en/schemas/cirnetrules-task.schema.json
    DESTINATION "${documentation_dir}/en/schemas")
foreach(language IN ITEMS en vi)
    install(FILES
        "${language}/references/dashboard.md"
        "${language}/references/dashboard-architecture.md"
        "${language}/references/dashboard-release.md"
        "${language}/references/governance-workspace.md"
        "${language}/references/adapter-capabilities.md"
        "${language}/references/cirnetrules-identity.md"
        "${language}/references/cirnetrules-local.md"
        "${language}/references/desktop-interaction.md"
        DESTINATION "${documentation_dir}/${language}/references")
    install(FILES "${language}/CHANGELOG.md" DESTINATION "${documentation_dir}/${language}")
endforeach()

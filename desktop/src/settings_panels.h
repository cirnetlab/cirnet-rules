#pragma once

#include "runtime_client.h"

#include <QJsonArray>

class QWidget;

using SettingsRequest = std::function<void(const QString &, QJsonObject, RuntimeClient::Reply)>;
using DeleteChat = std::function<void(const QString &, std::function<void(bool)>)>;

QWidget *makeStorageSettings(RuntimeClient &client, SettingsRequest request, std::function<void()> openArchived);
QWidget *makeArchivedChats(SettingsRequest request, std::function<QJsonArray()> bindings,
                           std::function<void()> conversationsChanged, DeleteChat deleteChat);

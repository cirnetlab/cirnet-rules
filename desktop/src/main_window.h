#pragma once

#include "runtime_client.h"

#include <QJsonArray>
#include <QMainWindow>

class QApplication;
class QCloseEvent;
class QComboBox;
class QFrame;
class QLabel;
class QLineEdit;
class QListWidget;
class QPlainTextEdit;
class QPushButton;
class QStackedWidget;
class QStandardItemModel;
class QTableView;
class QTimer;
class QVBoxLayout;

void applyTheme(QApplication &app);

class MainWindow final : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(RuntimeClient &client, QWidget *parent = nullptr);

protected:
    void closeEvent(QCloseEvent *event) override;
    void resizeEvent(QResizeEvent *event) override;

private:
    void setConnection(bool ready, const QString &status);
    void showPage(QWidget *page);
    void call(const QString &method, QJsonObject params, RuntimeClient::Reply reply);
    void showStatus(const QString &text, bool error = false);
    void showError(const QString &text);
    void about();

    QWidget *makeRepositories();
    void refreshRepositories();
    void appendRepositories();
    void updateRepositoryActions();
    QJsonObject repositorySelection() const;
    void chooseRepository(bool chooseFile, const QJsonObject &existing = {});
    void previewRepository(const QString &path, const QJsonObject &existing = {});
    void renameRepository();
    void archiveRepository();
    void selectRepository();

    QWidget *makeConversations();
    void refreshConversations(bool append = false);
    void newConversation();
    void openConversation(const QString &id);
    void editConversation(bool archive);
    void sendMessage();
    void refreshConversationEvents();
    void conversationTaskDetails();
    void cancelConversationTask();

    QWidget *makeAutomations();
    void refreshAutomations(bool append = false);
    void refreshTemplates();
    void renderAutomationCards();
    void editAutomation(const QJsonObject &existing = {}, const QJsonObject &workflow = {});
    void runAutomation(const QJsonObject &automation);
    void toggleAutomation(const QJsonObject &automation);
    void automationHistory(const QJsonObject &automation);

    RuntimeClient &client_;
    bool online_ = false;
    bool shutdownPending_ = false;
    QLabel *connection_ = nullptr;
    QLabel *feedback_ = nullptr;
    QPushButton *reconnect_ = nullptr;
    QStackedWidget *pages_ = nullptr;
    QWidget *repositoriesPage_ = nullptr;
    QWidget *conversationsPage_ = nullptr;
    QWidget *automationsPage_ = nullptr;
    QJsonObject runtimeInfo_;

    QTableView *repositoryTable_ = nullptr;
    QStandardItemModel *repositoryModel_ = nullptr;
    QPushButton *repoRelink_ = nullptr;
    QPushButton *repoRename_ = nullptr;
    QPushButton *repoArchive_ = nullptr;
    QPushButton *repoSelect_ = nullptr;
    QPushButton *repoMore_ = nullptr;
    QJsonArray repositories_;
    QJsonValue repoCursor_;
    bool includeArchived_ = false;
    quint64 repoRequest_ = 0;

    QLineEdit *conversationSearch_ = nullptr;
    QListWidget *conversationList_ = nullptr;
    QPushButton *conversationMore_ = nullptr;
    QPushButton *conversationRename_ = nullptr;
    QPushButton *conversationArchive_ = nullptr;
    QPushButton *cancelTask_ = nullptr;
    QPushButton *send_ = nullptr;
    QLabel *conversationTitle_ = nullptr;
    QLabel *conversationDetail_ = nullptr;
    QPlainTextEdit *transcript_ = nullptr;
    QPlainTextEdit *composer_ = nullptr;
    QJsonObject conversation_;
    QJsonArray conversationTasks_;
    QJsonArray conversationBindings_;
    QJsonValue conversationCursor_;
    QString currentConversation_;
    QString retryContent_;
    QString retryMessageKey_;
    quint64 conversationRequest_ = 0;
    quint64 detailRequest_ = 0;
    quint64 eventCursor_ = 0;

    QLineEdit *automationSearch_ = nullptr;
    QComboBox *automationFilter_ = nullptr;
    QComboBox *automationRepoFilter_ = nullptr;
    QVBoxLayout *automationCards_ = nullptr;
    QPushButton *automationMore_ = nullptr;
    QJsonArray automations_;
    QJsonArray templates_;
    QJsonValue automationCursor_;
    quint64 automationRequest_ = 0;
    int cardColumns_ = 0;
    QHash<QString, QString> runKeys_;
    QTimer *poll_ = nullptr;
};

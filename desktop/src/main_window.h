#pragma once

#include "runtime_client.h"

#include <QHash>
#include <QJsonArray>
#include <QMainWindow>
#include <QSet>
#include <QVector>

class QAction;
class QCheckBox;
class QCloseEvent;
class QComboBox;
class QFrame;
class QHBoxLayout;
class QLabel;
class QLineEdit;
class QListWidget;
class QPlainTextEdit;
class QPushButton;
class QScrollArea;
class QSortFilterProxyModel;
class QSpinBox;
class QSplitter;
class QStackedWidget;
class QStandardItemModel;
class QTableView;
class QTimer;
class QToolButton;
class QVBoxLayout;
class NotificationCenter;
class ThemeManager;

#include "theme.h"

class MainWindow final : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(RuntimeClient &client, QWidget *parent = nullptr);

protected:
    void closeEvent(QCloseEvent *event) override;

private:
    void setConnection(bool ready, const QString &status);
    void showPage(QWidget *page);
    void showSettings(const QString &section = {});
    void closeSettings();
    QWidget *makeSettings();
    void updateRuntimeDetails();
    void reconnectRuntime();
    void changeSidebar(bool collapsed);
    void updateShellIcons();
    void loadPreferences();
    void schedulePreferencesSave();
    void savePreferences();
    void applyPreferences(const QJsonObject &settings);
    QJsonObject preferenceSettings() const;
    void stopRuntimeAndClose();
    void call(const QString &method, QJsonObject params, RuntimeClient::Reply reply);
    void showStatus(const QString &text, bool error = false);
    void showError(const QString &text);

    QWidget *makeRepositories();
    void refreshRepositories();
    void appendRepositories();
    void fetchRepositoryPage();
    void renderRepositoryRows();
    void applyRepositoryFilter();
    void restoreRepositoryScroll();
    void ensureRepositoryCatalog(std::function<void(bool)> done);
    QJsonObject repositoryLayout() const;
    void restoreRepositoryLayout(const QJsonObject &layout);
    void checkRepository(bool openFolder);
    void updateRepositoryActions();
    QJsonObject repositorySelection() const;
    void chooseRepository(bool chooseFile, const QJsonObject &existing = {});
    void previewRepository(const QString &path, const QJsonObject &existing = {});
    void confirmRepository(const QString &path, const QJsonObject &existing, const QJsonObject &preview);
    void renameRepository();
    void archiveRepository();
    void selectRepository();

    QWidget *makeConversations();
    QWidget *makeConversationHistory();
    bool canLoadMoreConversations() const;
    void refreshConversations(bool append = false);
    void newConversation(const QString &preselected = {});
    void openConversation(const QString &id);
    void editConversation(bool archive, const QJsonObject &record = {});
    void updateConversationSummary();
    void updateConversationAvailability();
    void sendMessage();
    void refreshConversationEvents();
    void conversationTaskDetails();
    void cancelConversationTask();

    QWidget *makeAutomations();
    void refreshAutomations(bool append = false);
    void renderAutomationCards();
    void editAutomation(const QJsonObject &existing = {});
    void chooseAutomationFilter();
    void updateAutomationAvailability();
    void runAutomation(const QJsonObject &automation);
    void toggleAutomation(const QJsonObject &automation);
    void automationHistory(const QJsonObject &automation);

    RuntimeClient &client_;
    bool online_ = false;
    bool connecting_ = true;
    bool shutdownPending_ = false;
    bool closeAfterSave_ = false;
    QString connectionStatus_;
    ThemeManager *theme_ = nullptr;
    NotificationCenter *notifications_ = nullptr;
    QSplitter *splitter_ = nullptr;
    QFrame *sidebar_ = nullptr;
    QLabel *brand_ = nullptr;
    QLabel *brandSubtitle_ = nullptr;
    QHBoxLayout *identityLayout_ = nullptr;
    QHBoxLayout *localLayout_ = nullptr;
    QToolButton *sidebarToggle_ = nullptr;
    QToolButton *settingsButton_ = nullptr;
    QToolButton *localButton_ = nullptr;
    QPushButton *chatButton_ = nullptr;
    QPushButton *automationsButton_ = nullptr;
    QWidget *historyPage_ = nullptr;
    QAction *newChatAction_ = nullptr;
    int sidebarWidth_ = 260;
    bool sidebarCollapsed_ = false;
    bool applyingPreferences_ = false;
    bool preferencesLoaded_ = false;
    bool preferencesLoading_ = false;
    bool preferencesDirty_ = false;
    bool preferencesSaving_ = false;
    bool preferencesRetryRequired_ = false;
    qint64 preferencesRevision_ = 0;
    QJsonObject savedPreferences_;
    QTimer *preferencesTimer_ = nullptr;
    QComboBox *themeChoice_ = nullptr;
    QComboBox *densityChoice_ = nullptr;
    QSpinBox *textScale_ = nullptr;
    QComboBox *notificationChoice_ = nullptr;
    QLabel *preferencesStatus_ = nullptr;
    QPushButton *preferencesRetry_ = nullptr;
    QLabel *runtimeStatus_ = nullptr;
    QLabel *runtimeCapabilities_ = nullptr;
    QLabel *aboutDetails_ = nullptr;
    QPushButton *reconnect_ = nullptr;
    QStackedWidget *pages_ = nullptr;
    QWidget *settingsPage_ = nullptr;
    QWidget *previousPage_ = nullptr;
    QComboBox *settingsChoice_ = nullptr;
    QStackedWidget *settingsPages_ = nullptr;
    QWidget *repositoriesPage_ = nullptr;
    QWidget *conversationsPage_ = nullptr;
    QWidget *automationsPage_ = nullptr;
    QJsonObject runtimeInfo_;

    QTableView *repositoryTable_ = nullptr;
    QStandardItemModel *repositoryModel_ = nullptr;
    QSortFilterProxyModel *repositoryProxy_ = nullptr;
    QLineEdit *repoSearch_ = nullptr;
    QComboBox *repoStateFilter_ = nullptr;
    QCheckBox *repoArchived_ = nullptr;
    QLabel *repoSummary_ = nullptr;
    QLabel *repoDetails_ = nullptr;
    QAction *repoRelink_ = nullptr;
    QAction *repoRename_ = nullptr;
    QAction *repoArchive_ = nullptr;
    QAction *repoCheck_ = nullptr;
    QAction *repoOpen_ = nullptr;
    QAction *repoCopy_ = nullptr;
    QPushButton *repoSelect_ = nullptr;
    QPushButton *repoMore_ = nullptr;
    QJsonArray repositories_;
    QJsonValue repoCursor_;
    QJsonValue repoIncomingCursor_;
    bool repoLoading_ = false;
    bool repoComplete_ = false;
    bool repoRestoring_ = false;
    bool repoActionPending_ = false;
    int repoRefreshRows_ = 0;
    QJsonArray repoIncoming_;
    QVector<std::function<void(bool)>> repoWaiters_;
    QString repositorySelectedId_;
    int repositoryScroll_ = 0;
    int repositoryHScroll_ = 0;
    QHash<QString, QJsonObject> repositoryHealth_;
    quint64 repoRequest_ = 0;

    QLineEdit *conversationSearch_ = nullptr;
    QListWidget *conversationList_ = nullptr;
    QPushButton *conversationActions_ = nullptr;
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
    bool conversationLoading_ = false;
    QString currentConversation_;
    QHash<QString, QString> conversationDrafts_, conversationRetryKeys_, conversationRetryContents_;
    QSet<QString> conversationSending_;
    bool eventRequestPending_ = false;
    quint64 conversationRequest_ = 0;
    quint64 detailRequest_ = 0;
    quint64 eventCursor_ = 0;

    QLineEdit *automationSearch_ = nullptr;
    QComboBox *automationFilter_ = nullptr;
    QPushButton *automationRepoFilter_ = nullptr;
    QString automationBindingFilter_;
    QVBoxLayout *automationCards_ = nullptr;
    QScrollArea *automationScroll_ = nullptr;
    QLabel *automationState_ = nullptr;
    QPushButton *automationMore_ = nullptr;
    QJsonArray automations_;
    QHash<QString, QFrame *> automationCardWidgets_;
    QSet<QString> automationRunsPending_;
    QJsonValue automationCursor_;
    quint64 automationRequest_ = 0;
    int cardColumns_ = 0;
    QHash<QString, QString> runKeys_;
    QTimer *poll_ = nullptr;
};

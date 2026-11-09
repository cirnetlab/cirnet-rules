#pragma once

#include "runtime_client.h"

#include <QJsonArray>
#include <QWidget>
#include <functional>

class QCheckBox;
class QComboBox;
class QLabel;
class QLineEdit;
class QListWidget;
class QPlainTextEdit;
class QPushButton;
class QShowEvent;
class QResizeEvent;
class QScrollArea;
class QSplitter;
class QStandardItemModel;
class QTabWidget;
class QTableView;
class QTimer;
class GovernanceEditor;
class GovernancePreview;

class GovernancePanel final : public QWidget {
    Q_OBJECT

public:
    using Request = std::function<void(const QString &, QJsonObject, RuntimeClient::Reply)>;
    using Repositories = std::function<QJsonArray()>;

    GovernancePanel(Request request, Repositories repositories);
    void setAvailable(bool available);
    void repositoryCatalogChanged();
    bool confirmClose();

protected:
    void showEvent(QShowEvent *event) override;
    void resizeEvent(QResizeEvent *event) override;

private:
    QWidget *makeLibrary();
    QWidget *makeInspector();
    QWidget *makeAssignments();
    QWidget *makeEffective();
    QWidget *makeIntegrations();
    void refreshCatalog(bool append = false);
    void openAsset(const QString &id);
    void setAsset(const QJsonObject &asset);
    void refreshControls();
    void report(const QString &message, bool failure = false);
    void reportStatic(const char *message, bool failure = false);
    bool abandonEdits();
    void edited();
    QJsonObject document() const;
    QJsonObject versioned() const;
    void save();
    void validate();
    void compare(bool review);
    void publish();
    void showHistory();
    void createAsset(const QString &sourceName = {}, const QString &source = {});
    void changeAsset(const QString &action);
    void readImport(const std::function<void(const QString &, const QString &)> &accept);
    void editAttachment(bool add);
    void importBundle(const QString &source);
    void refreshScopes();
    void refreshAssignments();
    void updateAssignment(bool enabled);
    void previewEffective();
    void loadIntegrations();
    void probeIntegration();
    void inspectImport();
    void send(const QString &method, QJsonObject params, RuntimeClient::Reply reply);
    void mutate(const QString &method, QJsonObject params, RuntimeClient::Reply reply);
    void updateEditorPreview();
    void updateSelection();
    void updateAssetState();
    void layoutLibrary();
    void updateDiagnostics(const QJsonObject &result);

    Request request_;
    Repositories repositories_;
    bool online_ = false;
    bool inspecting_ = false;
    bool compact_ = false;
    QList<int> wideSizes_{240, 760};
    bool writing_ = false;
    bool dirty_ = false;
    bool loading_ = false;
    bool catalogReady_ = false;
    bool catalogLoading_ = false;
    bool assetLoading_ = false;
    bool integrationLoading_ = false;
    quint64 catalogGeneration_ = 0;
    quint64 assetGeneration_ = 0;
    quint64 scopeGeneration_ = 0;
    QJsonObject asset_;
    QJsonArray files_;
    QJsonArray adapterItems_;
    QJsonObject pendingMutation_;
    QString pendingMethod_;
    QString nextPage_;
    QString reviewToken_;
    QString selectedId_;

    const char *statusPhrase_ = nullptr;
    QLabel *status_ = nullptr;
    QLabel *diagnostics_ = nullptr;
    QTimer *autosave_ = nullptr;
    QTimer *filterTimer_ = nullptr;
    QTabWidget *pages_ = nullptr;
    QSplitter *librarySplit_ = nullptr;
    QComboBox *kind_ = nullptr;
    QCheckBox *archived_ = nullptr;
    QLineEdit *search_ = nullptr;
    QTableView *catalog_ = nullptr;
    QStandardItemModel *model_ = nullptr;
    QPushButton *more_ = nullptr;
    QWidget *catalogPanel_ = nullptr;
    QScrollArea *inspectorScroll_ = nullptr;
    QPushButton *backToCatalog_ = nullptr;
    QLabel *assetState_ = nullptr;
    QLineEdit *title_ = nullptr;
    QLineEdit *description_ = nullptr;
    QLineEdit *dependencies_ = nullptr;
    QComboBox *format_ = nullptr;
    QTabWidget *contentTabs_ = nullptr;
    GovernanceEditor *english_ = nullptr;
    GovernanceEditor *vietnamese_ = nullptr;
    QListWidget *fileList_ = nullptr;
    GovernancePreview *preview_ = nullptr;
    QComboBox *previewLocale_ = nullptr;
    QPlainTextEdit *diff_ = nullptr;
    QPushButton *new_ = nullptr;
    QPushButton *import_ = nullptr;
    QPushButton *save_ = nullptr;
    QPushButton *validate_ = nullptr;
    QPushButton *compare_ = nullptr;
    QPushButton *review_ = nullptr;
    QPushButton *apply_ = nullptr;
    QPushButton *history_ = nullptr;
    QPushButton *actions_ = nullptr;
    QComboBox *assignmentBinding_ = nullptr;
    QPlainTextEdit *assignmentResults_ = nullptr;
    QPushButton *assign_ = nullptr;
    QPushButton *unassign_ = nullptr;
    QComboBox *effectiveBinding_ = nullptr;
    QComboBox *effectiveMode_ = nullptr;
    QPlainTextEdit *effectiveResults_ = nullptr;
    QComboBox *adapter_ = nullptr;
    QLineEdit *port_ = nullptr;
    QPushButton *probe_ = nullptr;
    QPushButton *inspectImport_ = nullptr;
    QPlainTextEdit *adapterDetails_ = nullptr;
};

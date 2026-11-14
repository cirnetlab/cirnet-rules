const accounts = new Map([
  ['p-01', {
    displayName: 'Example One',
    theme: 'light',
    recoveryCode: 'FAKE_RECOVERY_DO_NOT_EXPORT_01',
    internalSupportNote: 'Synthetic support record A',
  }],
  ['p-02', {
    displayName: 'Example Two',
    theme: 'dark',
    recoveryCode: 'FAKE_RECOVERY_DO_NOT_EXPORT_02',
    internalSupportNote: 'Synthetic support record B',
  }],
]);

export function readPreferences(id) {
  const account = accounts.get(id);
  return account && { displayName: account.displayName, theme: account.theme };
}

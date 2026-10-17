// @vitest-environment jsdom
import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    refresh: vi.fn(),
    cached: vi.fn(),
    savedLogin: vi.fn(),
    manualLogin: vi.fn(),
    toast: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));
vi.mock('@/services/authSnapshotService', () => ({
    getCachedAuthSnapshot: mocks.cached,
    refreshSavedAuthSnapshot: mocks.refresh,
    deleteSavedAuthSnapshot: vi.fn()
}));
vi.mock('@/services/authExecutionService', () => ({
    executeManualLogin: mocks.manualLogin,
    executeSavedCredentialLogin: mocks.savedLogin,
    getAuthSnapshotFromExecutionError: () => null,
    showAuthFailureToast: mocks.toast
}));
vi.mock('@/services/authErrorDisplayService', () => ({
    getLoginErrorMessage: (error: Error) => error.message
}));
vi.mock('@/services/entityMediaService', () => ({ openExternalLink: vi.fn() }));
vi.mock('@/services/legacyVrcxMigrationService', () => ({
    promptLegacyVrcxForceMigration: vi.fn()
}));
vi.mock('@/services/preferencesService', () => ({
    loadPreferenceSnapshot: vi.fn(),
    setAppLanguagePreference: vi.fn()
}));
vi.mock('@/services/profileRestoreSelectionService', () => ({
    selectProfileBackupToRestore: vi.fn()
}));
vi.mock('@/services/proxySettingsService', () => ({
    proxySettingsErrorMessage: vi.fn(),
    saveProxySettingsPreferences: vi.fn(),
    testProxySettings: vi.fn()
}));
vi.mock('@/services/toastService', () => ({ toast: { add: mocks.toast } }));

import { useSessionStore } from '@/state/sessionStore';

import { useLoginPageState } from './useLoginPageState';

const account = {
    user: { id: 'usr_selected', displayName: 'Selected Account' },
    loginParams: { username: 'selected' },
    hasLoginCredentials: true,
    hasCookies: true
};
const snapshot = {
    lastUserLoggedIn: account.user.id,
    savedCredentialsList: [account],
    // A cached snapshot from a previous version must not enable automatic login.
    autoLoginStatus: 'available',
    autoLoginDelayEnabled: true,
    autoLoginDelaySeconds: 1
};

describe('manual account selection', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        vi.useFakeTimers();
        useSessionStore.getState().resetSessionState();
        useSessionStore.setState({
            databaseReady: true,
            sessionPhase: 'signed_out'
        });
        mocks.refresh.mockResolvedValue(snapshot);
        mocks.savedLogin.mockResolvedValue(snapshot);
    });
    afterEach(() => {
        cleanup();
        vi.useRealTimers();
    });

    it('only loads saved accounts on mount, after a delay and after remount', async () => {
        const first = renderHook(() => useLoginPageState());
        await act(async () => {
            await vi.advanceTimersByTimeAsync(60_000);
        });
        expect(first.result.current.savedAccounts).toEqual([account]);
        first.rerender();
        first.unmount();
        mocks.cached.mockReturnValue(snapshot);
        renderHook(() => useLoginPageState());
        await act(async () => {
            await vi.advanceTimersByTimeAsync(60_000);
        });
        expect(mocks.savedLogin).not.toHaveBeenCalled();
        expect(mocks.manualLogin).not.toHaveBeenCalled();
    });

    it('logs in exactly once after the user selects a saved account', async () => {
        const { result } = renderHook(() => useLoginPageState());
        await act(async () => {
            await result.current.handleSavedCredentialLogin(account);
        });
        expect(mocks.savedLogin).toHaveBeenCalledExactlyOnceWith(account);
        expect(mocks.manualLogin).not.toHaveBeenCalled();
    });

    it('does not retry a failed saved-account login without another user action', async () => {
        mocks.savedLogin.mockRejectedValue(new Error('Session expired'));
        const { result } = renderHook(() => useLoginPageState());
        await act(async () => {
            await result.current.handleSavedCredentialLogin(account);
        });
        await act(async () => {
            await vi.advanceTimersByTimeAsync(60_000);
        });
        expect(mocks.savedLogin).toHaveBeenCalledOnce();
        expect(mocks.toast).toHaveBeenCalled();
    });
});

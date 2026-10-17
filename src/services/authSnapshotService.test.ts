import { beforeEach, describe, expect, it, vi } from 'vitest';

const repositoryMocks = vi.hoisted(() => ({
    getSavedAuthSnapshot: vi.fn(),
    deleteSavedCredential: vi.fn()
}));

vi.mock('@/repositories/authRepository', () => ({
    default: repositoryMocks
}));

import type { SavedAuthSnapshot } from '@/repositories/authRepository';
import { useRuntimeStore } from '@/state/runtimeStore';

import {
    applySavedAuthSnapshot,
    deleteSavedAuthSnapshot,
    refreshSavedAuthSnapshot
} from './authSnapshotService';

function snapshot(patch: Partial<SavedAuthSnapshot> = {}): SavedAuthSnapshot {
    return {
        lastUserLoggedIn: 'usr_1',
        savedCredentialsList: [
            {
                user: { id: 'usr_1', displayName: 'User One' },
                loginParams: { username: 'user@example.test' },
                hasLoginCredentials: true,
                hasCookies: false
            }
        ],
        ...patch
    };
}

describe('authSnapshotService', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        useRuntimeStore.getState().resetRuntimeState();
        repositoryMocks.getSavedAuthSnapshot.mockResolvedValue(snapshot());
        repositoryMocks.deleteSavedCredential.mockResolvedValue(
            snapshot({
                lastUserLoggedIn: null,
                savedCredentialsList: []
            })
        );
    });

    it('applies an available saved-auth snapshot to runtime auth and startup state', () => {
        const applied = applySavedAuthSnapshot(snapshot());

        expect(applied).toMatchObject({
            lastUserLoggedIn: 'usr_1'
        });
        expect(useRuntimeStore.getState().auth).toMatchObject({
            lastUserLoggedIn: 'usr_1',
            savedCredentialCount: 1
        });
        expect(useRuntimeStore.getState().startup.auth).toMatchObject({
            status: 'completed',
            detail: 'Waiting for the user to sign in manually.'
        });
    });

    it('marks missing saved credentials as a completed auth startup task', () => {
        applySavedAuthSnapshot(
            snapshot({ lastUserLoggedIn: null, savedCredentialsList: [] })
        );

        expect(useRuntimeStore.getState().startup.auth).toMatchObject({
            status: 'completed',
            detail: 'Waiting for the user to sign in manually.'
        });
    });

    it('refreshes and deletes snapshots through authRepository before applying them', async () => {
        await expect(refreshSavedAuthSnapshot()).resolves.toMatchObject({
            lastUserLoggedIn: 'usr_1'
        });
        await expect(deleteSavedAuthSnapshot('usr_1')).resolves.toMatchObject({
            savedCredentialsList: []
        });

        expect(repositoryMocks.getSavedAuthSnapshot).toHaveBeenCalledTimes(1);
        expect(repositoryMocks.deleteSavedCredential).toHaveBeenCalledWith(
            'usr_1'
        );
        expect(useRuntimeStore.getState().auth).toMatchObject({
            lastUserLoggedIn: null,
            savedCredentialCount: 0
        });
    });
});

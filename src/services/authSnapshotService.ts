import authRepository, {
    type SavedAuthSnapshot
} from '@/repositories/authRepository';
import { useRuntimeStore } from '@/state/runtimeStore';

let cachedAuthSnapshot: SavedAuthSnapshot | null = null;

export function applySavedAuthSnapshot(
    snapshot: SavedAuthSnapshot
): SavedAuthSnapshot {
    const runtimeStore = useRuntimeStore.getState();
    runtimeStore.setAuthBootstrap({
        lastUserLoggedIn: snapshot.lastUserLoggedIn,
        savedCredentialCount: snapshot.savedCredentialsList.length
    });

    runtimeStore.setStartupTask(
        'auth',
        'completed',
        'Waiting for the user to sign in manually.'
    );
    cachedAuthSnapshot = snapshot;
    return snapshot;
}

export async function refreshSavedAuthSnapshot() {
    const snapshot = await authRepository.getSavedAuthSnapshot();
    return applySavedAuthSnapshot(snapshot);
}

export function getCachedAuthSnapshot(): SavedAuthSnapshot | null {
    return cachedAuthSnapshot;
}

export async function deleteSavedAuthSnapshot(userId: string) {
    const snapshot = await authRepository.deleteSavedCredential(userId);
    return applySavedAuthSnapshot(snapshot);
}

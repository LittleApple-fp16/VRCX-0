// @vitest-environment jsdom

import {
    cleanup,
    fireEvent,
    render,
    screen,
    waitFor
} from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

import {
    commands,
    type HighRiskDetectionSettings
} from '@/platform/tauri/bindings';
import { useRuntimeStore } from '@/state/runtimeStore';

import { HighRiskDetectionSettingsCard } from './HighRiskDetectionSettingsCard';

const initialCapabilities = useRuntimeStore.getState().hostCapabilities;
const defaults: HighRiskDetectionSettings = {
    rooms: ['public', 'groupPublic', 'groupPlus'],
    nickname: 'english',
    ranks: ['newUser', 'visitor'],
    marks: ['probableTroll', 'troll'],
    autoBlock: true,
    shortcut: { key: 'KeyF', control: true, alt: false, shift: false }
};

afterEach(() => {
    cleanup();
    useRuntimeStore.setState({ hostCapabilities: initialCapabilities });
    vi.restoreAllMocks();
});

it('saves room and mark selections independently', async () => {
    useRuntimeStore.setState({
        hostCapabilities: { ...initialCapabilities, platform: 'windows' }
    });
    vi.spyOn(commands, 'appGetHighRiskDetection').mockResolvedValue(defaults);
    const save = vi
        .spyOn(commands, 'appSetHighRiskDetection')
        .mockResolvedValue(null);

    render(<HighRiskDetectionSettingsCard />);

    const friends = await screen.findByRole('checkbox', {
        name: 'high_risk.options.friends'
    });
    expect(friends.getAttribute('aria-checked')).toBe('false');
    fireEvent.click(friends);

    const withFriends = { ...defaults, rooms: [...defaults.rooms, 'friends'] };
    await waitFor(() =>
        expect(friends.getAttribute('aria-checked')).toBe('true')
    );
    expect(save).toHaveBeenNthCalledWith(1, withFriends);

    const mark = screen.getByRole('checkbox', {
        name: 'high_risk.options.probableTroll'
    });
    expect(mark.getAttribute('aria-checked')).toBe('true');
    expect(
        screen
            .getByRole('checkbox', { name: 'high_risk.options.troll' })
            .getAttribute('aria-checked')
    ).toBe('true');
    fireEvent.click(mark);

    await waitFor(() =>
        expect(mark.getAttribute('aria-checked')).toBe('false')
    );
    expect(save).toHaveBeenNthCalledWith(2, {
        ...withFriends,
        marks: ['troll']
    });
});

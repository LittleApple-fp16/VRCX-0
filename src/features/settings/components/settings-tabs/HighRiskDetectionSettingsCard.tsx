import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { KeyboardShortcut } from '@/components/keyboard/KeyboardShortcut';
import { trayShortcutKeys } from '@/components/keyboard/trayShortcutKeys';
import {
    commands,
    type HighRiskDetectionSettings,
    type HighRiskMark,
    type HighRiskNickname,
    type HighRiskRank,
    type HighRiskRoom,
    type TrayShortcutError
} from '@/platform/tauri/bindings';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Button } from '@/ui/shadcn/button';
import { Checkbox } from '@/ui/shadcn/checkbox';
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue
} from '@/ui/shadcn/select';
import { Switch } from '@/ui/shadcn/switch';

import { SettingsCard } from '../SettingsCard';
import { Field } from '../SettingsField';
import { GlobalShortcutRecorder } from '../TrayShortcutSetting';

const rooms: HighRiskRoom[] = [
    'public',
    'groupPublic',
    'groupPlus',
    'friendsPlus',
    'friends',
    'invitePlus',
    'invite'
];
const ranks: HighRiskRank[] = [
    'visitor',
    'newUser',
    'user',
    'knownUser',
    'trustedUser'
];
const marks: HighRiskMark[] = ['probableTroll', 'troll'];
const nicknames: HighRiskNickname[] = ['english', 'suffix', 'ignore'];

type ChoiceField = 'rooms' | 'ranks' | 'marks';

function Choices<T extends string>({
    settingId,
    field,
    options,
    value,
    disabled,
    onChange
}: {
    settingId: string;
    field: ChoiceField;
    options: T[];
    value: T[];
    disabled: boolean;
    onChange: (value: T[]) => void;
}) {
    const { t } = useTranslation();
    const label = t(`high_risk.${field}`);
    return (
        <Field settingId={settingId} label={label}>
            <div
                role="group"
                aria-label={label}
                className="grid w-60 grid-cols-2 gap-2"
            >
                {options.map((option) => (
                    <label
                        key={option}
                        className="flex items-center gap-2 text-sm"
                    >
                        <Checkbox
                            checked={value.includes(option)}
                            disabled={disabled}
                            onCheckedChange={(checked) =>
                                onChange(
                                    checked
                                        ? [...value, option]
                                        : value.filter(
                                              (item) => item !== option
                                          )
                                )
                            }
                        />
                        {t(`high_risk.options.${option}`)}
                    </label>
                ))}
            </div>
        </Field>
    );
}

export function HighRiskDetectionSettingsCard() {
    const windows = useRuntimeStore(
        (state) => state.hostCapabilities.platform === 'windows'
    );
    return windows ? <HighRiskDetectionSettingsFields /> : null;
}

function HighRiskDetectionSettingsFields() {
    const { t } = useTranslation();
    const [settings, setSettings] = useState<HighRiskDetectionSettings | null>(
        null
    );
    const [saving, setSaving] = useState(false);
    const [loadError, setLoadError] = useState(false);
    const [saveError, setSaveError] = useState(false);
    const [editing, setEditing] = useState(false);
    const [shortcutError, setShortcutError] =
        useState<TrayShortcutError | null>(null);

    useEffect(() => {
        let active = true;
        void commands
            .appGetHighRiskDetection()
            .then((result) => {
                if (!active) return;
                setSettings(result);
                setLoadError(result === null);
            })
            .catch(() => {
                if (active) setLoadError(true);
            });
        return () => {
            active = false;
        };
    }, []);

    async function save(next: HighRiskDetectionSettings) {
        setSaving(true);
        setSaveError(false);
        try {
            const error = await commands.appSetHighRiskDetection(next);
            if (error) {
                setShortcutError(error);
                return;
            }
            setSettings(next);
            setEditing(false);
        } catch {
            if (editing) {
                setShortcutError('unavailable');
            } else {
                setSaveError(true);
            }
        } finally {
            setSaving(false);
        }
    }

    return (
        <SettingsCard
            cardId="notifications.high-risk"
            title={t('high_risk.title')}
            description={t('high_risk.description')}
        >
            {settings ? (
                <>
                    <Choices
                        settingId="notifications.high-risk.rooms"
                        field="rooms"
                        options={rooms}
                        value={settings.rooms}
                        disabled={saving}
                        onChange={(value) =>
                            void save({ ...settings, rooms: value })
                        }
                    />
                    <Field
                        settingId="notifications.high-risk.nickname"
                        label={t('high_risk.nickname')}
                        description={t('high_risk.nickname_description')}
                        controlId="settings-high-risk-nickname"
                    >
                        <Select
                            value={settings.nickname}
                            items={nicknames.map((value) => ({
                                value,
                                label: t(`high_risk.options.${value}`)
                            }))}
                            disabled={saving}
                            onValueChange={(value) => {
                                if (value)
                                    void save({ ...settings, nickname: value });
                            }}
                        >
                            <SelectTrigger
                                id="settings-high-risk-nickname"
                                className="w-60"
                            >
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectGroup>
                                    {nicknames.map((value) => (
                                        <SelectItem key={value} value={value}>
                                            {t(`high_risk.options.${value}`)}
                                        </SelectItem>
                                    ))}
                                </SelectGroup>
                            </SelectContent>
                        </Select>
                    </Field>
                    <Choices
                        settingId="notifications.high-risk.ranks"
                        field="ranks"
                        options={ranks}
                        value={settings.ranks}
                        disabled={saving}
                        onChange={(value) =>
                            void save({ ...settings, ranks: value })
                        }
                    />
                    <Choices
                        settingId="notifications.high-risk.marks"
                        field="marks"
                        options={marks}
                        value={settings.marks}
                        disabled={saving}
                        onChange={(value) =>
                            void save({ ...settings, marks: value })
                        }
                    />
                    <Field
                        settingId="notifications.high-risk.auto-block"
                        label={t('high_risk.auto_block')}
                    >
                        <Switch
                            checked={settings.autoBlock}
                            disabled={saving}
                            onCheckedChange={(value) =>
                                void save({ ...settings, autoBlock: value })
                            }
                        />
                    </Field>
                    <Field
                        settingId="notifications.high-risk.shortcut"
                        label={t('high_risk.shortcut')}
                        description={t('high_risk.shortcut_description')}
                    >
                        <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            disabled={saving}
                            aria-label={t('high_risk.shortcut')}
                            onClick={() => {
                                setShortcutError(null);
                                setEditing(true);
                            }}
                        >
                            <KeyboardShortcut
                                keys={trayShortcutKeys(settings.shortcut)}
                            />
                        </Button>
                    </Field>
                    {editing ? (
                        <GlobalShortcutRecorder
                            title={t('high_risk.shortcut')}
                            initialBinding={settings.shortcut}
                            pending={saving}
                            error={shortcutError}
                            checkBinding={commands.appCheckHighRiskShortcut}
                            onErrorChange={setShortcutError}
                            onClose={() => {
                                if (!saving) setEditing(false);
                            }}
                            onSave={(shortcut) =>
                                save({ ...settings, shortcut })
                            }
                        />
                    ) : null}
                </>
            ) : !loadError ? (
                <p role="status" className="py-2 text-sm">
                    {t('common.loading')}
                </p>
            ) : null}
            {loadError || saveError ? (
                <p role="alert" className="text-destructive py-2 text-sm">
                    {t(
                        loadError
                            ? 'high_risk.load_error'
                            : 'high_risk.save_error'
                    )}
                </p>
            ) : null}
        </SettingsCard>
    );
}

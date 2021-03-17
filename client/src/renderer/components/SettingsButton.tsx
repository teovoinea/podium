import * as React from 'react';
// npm install --save-dev @iconify/react @iconify-icons/clarity
import { Icon, InlineIcon } from '@iconify/react';
import settingsSolid from '@iconify-icons/clarity/settings-solid';

require('./SettingsButton.scss');
export interface Props {}

const SettingsButton: React.FunctionComponent<Props> = () => (
    <div className="settingsButton">
        <Icon icon={settingsSolid} />
    </div>
);

export default SettingsButton;

import './style.css';

import map from './modules/map.mjs';
import mode_control from './modules/mode_control.mjs';
import cost_control from './modules/cost_control.mjs';

mode_control.render(map);
cost_control.render(map);

import { useState } from 'react';
import { Settings as SettingsIcon, Save, Key, Cpu, Sparkles } from 'lucide-react';

export const Settings = () => {
    const [apiKey, setApiKey] = useState(localStorage.getItem('jacox_api_key') || 'sk-dev-key-123');

    const handleSave = () => {
        localStorage.setItem('jacox_api_key', apiKey);
        alert('Settings saved!');
    };

    return (
        <div className="flex flex-col gap-8">
            <header>
                <h1 className="text-4xl font-bold flex items-center gap-3">
                    <SettingsIcon className="text-gruv-yellow" />
                    Settings
                </h1>
                <p className="text-gruv-light-4">Configure your Jacox instance and LLM providers.</p>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="glass p-8 rounded-2xl flex flex-col gap-6">
                    <h3 className="text-xl font-bold flex items-center gap-2">
                        <Key className="text-monokai-pink w-5 h-5" />
                        Authentication
                    </h3>
                    <div className="flex flex-col gap-2">
                        <label className="text-xs font-mono text-gruv-light-4 uppercase">Jacox API Key</label>
                        <input
                            type="password"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            className="bg-gruv-dark-3 border border-gruv-dark-4 rounded-xl py-3 px-4 focus:outline-none focus:border-monokai-pink"
                        />
                        <p className="text-[10px] text-gruv-light-4 leading-relaxed mt-1">
                            Required for all local API calls. Default is <code>sk-dev-key-123</code>.
                        </p>
                    </div>
                    <button onClick={handleSave} className="btn-primary w-full mt-2 flex items-center justify-center gap-2">
                        <Save className="w-4 h-4" />
                        Save Configuration
                    </button>
                </div>

                <div className="glass p-8 rounded-2xl flex flex-col gap-6">
                    <h3 className="text-xl font-bold flex items-center gap-2">
                        <Cpu className="text-monokai-aqua w-5 h-5" />
                        Agent Interface
                    </h3>
                    <div className="space-y-4">
                        <div className="flex justify-between items-center p-4 bg-gruv-dark-3/50 rounded-xl border border-gruv-dark-4/20">
                            <div>
                                <p className="text-sm font-bold">Provider</p>
                                <p className="text-xs text-monokai-aqua uppercase font-mono">Ollama (Native)</p>
                            </div>
                            <Sparkles className="text-monokai-purple w-5 h-5" />
                        </div>
                        <div className="flex justify-between items-center p-4 bg-gruv-dark-3/50 rounded-xl border border-gruv-dark-4/20 opacity-50">
                            <div>
                                <p className="text-sm font-bold">Default Model</p>
                                <p className="text-xs text-gruv-light-4">Configure in config.yaml</p>
                            </div>
                        </div>
                    </div>
                    <p className="text-xs text-gruv-light-4 p-4 bg-gruv-yellow/5 border border-gruv-yellow/20 rounded-xl">
                        Currently, provider switching is managed via the <code>config.yaml</code> on the server. Frontend controls are coming in V0.2.
                    </p>
                </div>
            </div>
        </div>
    );
};

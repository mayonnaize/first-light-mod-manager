$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

& "$PSScriptRoot\..\node_modules\.bin\tauri.cmd" @args
exit $LASTEXITCODE

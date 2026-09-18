const TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Configuration status="WARN">
    <Appenders>
        <Console name="SysOut" target="SYSTEM_OUT">
            <PatternLayout pattern="[%level] [%t]: %msg{nolookups}%n" />
        </Console>
        <RollingRandomAccessFile name="File" fileName="logs/latest.log" filePattern="logs/%d{yyyy-MM-dd}-%i.log.gz">
            <PatternLayout pattern="[%d{HH:mm:ss}] [%t/%level]: %msg{nolookups}%n" />
            <Policies>
                <TimeBasedTriggeringPolicy />
                <OnStartupTriggeringPolicy />
            </Policies>
        </RollingRandomAccessFile>
    </Appenders>
    <Loggers>
        <Root level="{level}">
            <filters>
                <MarkerFilter marker="NETWORK_PACKETS" onMatch="DENY" onMismatch="NEUTRAL" />
            </filters>
            <AppenderRef ref="SysOut" level="{level}" />
            <AppenderRef ref="File" level="info" />
        </Root>
    </Loggers>
</Configuration>
"#;

// The console never drops below info: mc reads it to recognize events, and
// filters what it echoes against its own verbosity.
pub fn configuration(level: &str) -> String {
    let level = match level {
        "debug" | "trace" => level,
        _ => "info"
    };

    TEMPLATE.replace("{level}", level)
}

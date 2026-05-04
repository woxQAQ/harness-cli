package sample

type Reader interface {
    Read(key string) string
    Close() error
}

type Handler func(string) string

type RetryPolicy struct {
    MaxAttempts int
}

type Config struct {
    Policy RetryPolicy
    TimeoutSeconds int
}

type Service struct {
    reader Reader
    config Config
}

func NewService(reader Reader, config Config) Service {
    return Service{reader: reader, config: config}
}

func (s Service) Run(key string) string {
    return s.reader.Read(key)
}

func (s Service) Close() error {
    return s.reader.Close()
}

func defaultConfig() Config {
    return Config{
        Policy: RetryPolicy{MaxAttempts: 3},
        TimeoutSeconds: 5,
    }
}

func bootstrap(reader Reader) Service {
    config := defaultConfig()
    var fallback = func(key string) string {
        return reader.Read(key)
    }
    helper := func(key string) string {
        inner := func(value string) string {
            return fallback(value)
        }
        return inner(key)
    }
    _ = helper("warmup")
    return NewService(reader, config)
}

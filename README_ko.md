# PVM - Python Version Manager

🇺🇸 [English](README.md) | 🇰🇷 [한국어](README_ko.md)

Rust로 작성된 경량 독립형 Python 버전 및 가상 환경 관리자. Anaconda처럼 쓰지만, 빠르고 간단합니다.

## 왜 PVM인가?

| 기능 | PVM | uv/mise | Anaconda |
|------|-----|---------|----------|
| 공유 환경 | ✅ | ❌ (프로젝트별) | ✅ |
| 패키지 중복 제거 | ✅ (하드링크) | ✅ | ❌ |
| 외부 의존성 없음 | ✅ | ❌ | ❌ |
| 단일 바이너리 | ✅ (2.6MB) | ✅ | ❌ |
| 빠른 속도 | ✅ | ✅ | ❌ |

**핵심 차별점**: 하나의 Python 버전으로 여러 가상 환경을 생성하고, 여러 프로젝트에서 사용할 수 있습니다. 각 프로젝트 디렉토리에 `.venv`를 생성하는 도구와 달리, PVM은 환경을 중앙에서 관리합니다—Anaconda처럼, 하지만 무겁지 않게.

## 설치

### 빠른 설치 (권장)

플랫폼에 맞는 최신 사전 빌드 바이너리를 내려받고 SHA256 체크섬을 검증한 뒤 `~/.pvm`에 설치합니다.

```bash
curl -fsSL https://pvm.sungjin.dev/install.sh | bash
```

바이너리는 `~/.local/bin/pvm`에, 상태 디렉토리는 `~/.pvm/`에 설치됩니다. 경로는 `PVM_BIN_DIR` / `PVM_HOME`으로 변경할 수 있습니다.

이후 셸 통합(activate/deactivate + 자동 완성 + legacy alias)을 활성화하세요:

```bash
# zsh
echo 'eval "$(pvm init zsh)"' >> ~/.zshrc && eval "$(pvm init zsh)"

# bash
echo 'eval "$(pvm init bash)"' >> ~/.bashrc && eval "$(pvm init bash)"
```

`~/.local/bin`이 `PATH`에 없으면 `export PATH="$HOME/.local/bin:$PATH"`도 셸 rc에 추가하세요.

설치 상태 점검:

```bash
pvm doctor
```

**대화형 프롬프트 건너뛰기** (기본값 사용):

```bash
curl -fsSL https://pvm.sungjin.dev/install.sh | bash -s -- --yes
```

**특정 버전 고정:**

```bash
curl -fsSL https://pvm.sungjin.dev/install.sh | PVM_VERSION=v0.1.0 bash
```

지원 플랫폼: macOS (Apple Silicon / Intel), Linux (x86_64 / aarch64).

### 소스에서 빌드

Rust 툴체인이 필요합니다 (edition 2021).

```bash
git clone https://github.com/taintlesscupcake/pvm.git
cd pvm
cargo build --release
./scripts/install.sh
eval "$(pvm init zsh)"   # 또는: pvm init bash
```

## 빠른 시작

```bash
# Python 설치
pvm python install 3.12

# 환경 생성
pvm env create myproject 3.12

# 환경 활성화
pvm env activate myproject

# 패키지 설치 - pip이 자동으로 중복 제거 기능과 연동됩니다!
pip install requests numpy pandas

# 작업 완료 후 비활성화
pvm env deactivate
```

## 명령어

### Python 관리

```bash
pvm python install <version>    # Python 설치 (예: 3.12, 3.11.9, 3.8)
pvm python list                 # 설치된 버전 목록
pvm python available            # 사용 가능한 버전 표시 (3.8 - 3.14)
pvm python remove <version>     # 버전 삭제
pvm update                      # Python 버전 메타데이터 새로고침
```

### 환경 관리

```bash
pvm env create <name> [version] # 환경 생성 (버전 미지정 시 대화형)
pvm env list                    # 모든 환경 목록
pvm env activate <name>         # 환경 활성화
pvm env deactivate              # 현재 환경 비활성화
pvm env remove <name>           # 환경 삭제
```

### 패키지 관리 (중복 제거 포함)

pvm 환경이 활성화되면, `pip install`이 자동으로 중복 제거 기능을 사용하도록 래핑됩니다:

```bash
# 먼저 환경 활성화
pvm env activate myproject

# pip install이 자동으로 중복 제거를 사용합니다!
pip install requests numpy           # → pvm pip install로 라우팅
pip install -r requirements.txt      # 모든 pip install 옵션 지원

# 다른 pip 명령어는 정상 동작
pip uninstall requests               # → 일반 pip 사용
pip freeze                           # → 일반 pip 사용
pip list                             # → 일반 pip 사용
```

`pvm pip`을 명시적으로 사용할 수도 있습니다:

```bash
pvm pip install <packages>           # 중복 제거와 함께 설치
pvm pip install -r requirements.txt  # 모든 pip 옵션 지원
pvm pip sync                         # 기존 패키지 중복 제거

# 환경 활성화 없이 환경 지정
pvm pip install -e <env> <packages>
pvm pip sync -e <env>
```

### 캐시 관리

```bash
pvm cache info                  # 캐시 통계 표시
pvm cache list                  # 캐시된 패키지 목록
pvm cache savings               # 디스크 공간 절약량 표시
pvm cache clean                 # 고아 패키지 삭제
```

### 설정

```bash
pvm config show                 # 현재 설정 표시
pvm config get <key>            # 설정값 조회
pvm config set <key> <value>    # 설정값 변경
pvm config sync                 # config.toml에서 shell.conf 재생성
pvm config reset                # 설정 초기화
```

사용 가능한 설정 키:
- `shell.legacy_commands` - 레거시 별칭 활성화 (기본값: true)
- `shell.pip_wrapper` - 자동 pip 래핑 활성화 (기본값: true)
- `general.auto_update_days` - 메타데이터 자동 업데이트 주기 (기본값: 7)
- `general.colored_output` - 컬러 출력 활성화 (기본값: true)
- `dedup.enabled` - 패키지 중복 제거 활성화 (기본값: true)
- `dedup.link_strategy` - 링크 전략: auto, hardlink, clone, copy (기본값: auto)

### 셸 자동 완성

PVM은 Bash와 Zsh에서 탭 자동 완성을 지원합니다:

```bash
# pvm.sh를 source하면 자동 완성이 자동으로 로드됩니다
source ~/.pvm/pvm.sh

# 또는 독립 완성 스크립트 생성
pvm completion bash > ~/.bash_completion.d/pvm
pvm completion zsh > ~/.zfunc/_pvm
```

지원되는 자동 완성:
- 명령어 및 하위 명령어
- 환경 이름 (`pvm env activate <TAB>`)
- Python 버전 (`pvm python install <TAB>`)
- 설정 키 및 값 (`pvm config set <TAB>`)

### 별칭

다른 도구에서 마이그레이션하는 사용자를 위해 레거시 별칭을 사용할 수 있습니다 (`pvm config set shell.legacy_commands false`로 비활성화 가능):

```bash
mkenv <version> <name>    # → pvm env create <name> <version>
rmenv <name>              # → pvm env remove <name>
lsenv                     # → pvm env list
act <name>                # → pvm env activate <name>
deact                     # → pvm env deactivate
```

### 외부 가상환경 마이그레이션

PVM은 외부 소스(예: virtualenvwrapper, mise 관리 가상환경)에서 기존 가상환경을 가져올 수 있습니다:

```bash
# 마이그레이션 가능한 환경 목록 표시
pvm migrate list
pvm migrate list --source /path/to/envs

# 단일 환경 마이그레이션
pvm migrate env myenv
pvm migrate env myenv --rename new-name    # 마이그레이션 중 이름 변경

# 모든 환경 일괄 마이그레이션
pvm migrate env --all

# 마이그레이션 후 소스 자동 삭제
pvm migrate env myenv --delete-source

# 비대화형 모드
pvm migrate env myenv -y --delete-source
```

**마이그레이션 과정:**
1. 소스 환경의 `pyvenv.cfg`에서 Python 버전 감지
2. 필요시 해당 Python 버전을 pvm에 자동 설치
3. 환경을 `~/.pvm/envs/`로 복사
4. Python 심볼릭 링크를 pvm 관리 Python으로 수정
5. `pvm pip sync` 실행하여 패키지를 캐시에 중복 제거
6. 소스 환경 삭제 여부 확인 (기본값: 유지)

**기본 소스:** `~/.virtualenvs/envs` (`--source`로 변경 가능)

## 디렉토리 구조

```
~/.pvm/
├── bin/pvm                 # PVM 바이너리
├── pvm.sh                  # 셸 통합
├── python-metadata.json    # 캐시된 버전 메타데이터 (자동 업데이트)
├── pythons/                # 설치된 Python 버전
│   ├── 3.12.4/
│   └── 3.11.9/
├── envs/                   # 가상 환경
│   ├── myproject/
│   └── datascience/
├── packages/               # 중복 제거된 패키지 캐시
│   ├── metadata.json       # 캐시 메타데이터
│   └── store/              # 내용 주소 지정 스토리지
└── cache/                  # 다운로드 캐시
```

## 작동 원리

1. **Python 설치**: [python-build-standalone](https://github.com/astral-sh/python-build-standalone) (uv/ruff 개발사 Astral이 관리)에서 사전 빌드된 Python을 다운로드합니다

2. **버전 메타데이터**: [uv의 download-metadata.json](https://github.com/astral-sh/uv)을 사용하여 Python 버전 (3.8 - 3.14)을 올바른 릴리스 태그에 매핑합니다. 메타데이터는 로컬에 캐시되며 7일마다 자동 업데이트됩니다. `pvm update`로 수동 새로고침할 수 있습니다.

3. **환경 생성**: Python 내장 `venv` 모듈을 사용하여 격리된 환경을 생성합니다

4. **활성화**: 셸 래퍼가 환경의 activate 스크립트를 source하고, `pip install`을 래핑하여 자동으로 중복 제거를 사용합니다

5. **패키지 중복 제거**: 패키지 설치 시 (활성화된 환경에서 `pip install` 또는 `pvm pip install` 사용), 여러 환경에서 동일한 패키지가 전역 캐시에 한 번만 저장되고 각 환경의 site-packages에 하드링크됩니다. NumPy, PyTorch 등 공통 패키지를 여러 환경에서 공유할 때 상당한 디스크 공간을 절약할 수 있습니다.

### 하드링크에 대한 중요 참고사항

패키지 중복 제거는 **하드링크**를 사용하여 캐시와 환경 간에 파일을 공유합니다. 이는 다음을 의미합니다:

- **공유 inode**: 여러 환경이 디스크의 동일한 파일을 가리킴
- **수정 전파**: 캐시된 패키지 파일을 수동으로 수정하면, 해당 파일을 사용하는 모든 환경에 영향
- **권장 사항**: 설치된 패키지 파일을 수동으로 편집하지 마세요. 다른 버전이 필요하면 `pip install --upgrade`를 사용하거나 새 환경을 생성하세요

## 개발

```bash
# 빌드
cargo build

# 테스트 실행
cargo test

# 릴리스 빌드
cargo build --release
```

### 프로젝트 구조

```
crates/
├── pvm-core/     # 코어 라이브러리 (다운로더, 설치 관리자, venv)
├── pvm-cli/      # CLI 애플리케이션
└── pvm-shell/    # 셸 통합
```

## 설정

설정은 `~/.pvm/config.toml`에 저장됩니다. `pvm config` 명령어로 설정을 관리하세요.

설치 디렉토리를 변경하려면 `PVM_HOME`을 설정하세요:

```bash
export PVM_HOME=/custom/path
source ~/.pvm/pvm.sh
```

## 라이선스

MIT

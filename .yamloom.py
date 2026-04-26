from __future__ import annotations

from yamloom import (
    Events,
    Job,
    PullRequestEvent,
    PushEvent,
    Workflow,
    WorkflowDispatchEvent,
    script,
)
from yamloom.actions.github.release import ReleasePlease
from yamloom.actions.github.scm import Checkout
from yamloom.actions.toolchains.rust import InstallRustTool, SetupRust
from yamloom.expressions import context

ci_workflow = Workflow(
    name='CI',
    on=Events(
        push=PushEvent(branches=['main']),
        pull_request=PullRequestEvent(opened=True, synchronize=True, reopened=True),
        workflow_dispatch=WorkflowDispatchEvent(),
    ),
    jobs={
        'check': Job(
            name='Check, lint, test, and document',
            runs_on='ubuntu-latest',
            steps=[
                Checkout(),
                SetupRust(components=['rustfmt', 'clippy']),
                script('cargo fmt --all --check'),
                script('cargo check --workspace --all-targets --all-features'),
                script('cargo clippy --workspace --all-targets --all-features -- -D warnings'),
                script('cargo test --workspace --all-features'),
                script('cargo doc --workspace --all-features --no-deps'),
            ],
            env={'RUSTDOCFLAGS': '-D warnings'},
        )
    },
)


release_please_workflow = Workflow(
    name='Release Please',
    on=Events(
        push=PushEvent(branches=['main']),
        workflow_dispatch=WorkflowDispatchEvent(),
    ),
    jobs={
        'release-please': Job(
            runs_on='ubuntu-latest',
            steps=[
                ReleasePlease(
                    id='release',
                    token=context.secrets.RELEASE_PLEASE,
                ),
                Checkout(condition=ReleasePlease.releases_created('release').from_json_to_bool()),
                SetupRust(condition=ReleasePlease.releases_created('release').from_json_to_bool()),
                InstallRustTool(
                    tool=['cargo-workspaces'],
                    condition=ReleasePlease.releases_created('release').from_json_to_bool(),
                ),
                script(
                    f'cargo workspaces publish --from-git --token {context.secrets.CARGO_REGISTRY_TOKEN} --yes',
                    condition=ReleasePlease.releases_created('release').from_json_to_bool(),
                ),
            ],
        )
    },
)

if __name__ == '__main__':
    ci_workflow.dump('.github/workflows/ci.yml')
    release_please_workflow.dump('.github/workflows/release-please.yml')

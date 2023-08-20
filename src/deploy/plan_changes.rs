use super::model;
use std::collections;
use std::iter;

pub fn go(
    actual_containers: &model::ActualContainers,
    desired_services: &model::DesiredServices,
) -> Vec<model::ServiceContainerChange> {
    let service_names = actual_containers
        .iter()
        .map(|container| &container.service_name)
        .chain(desired_services.keys())
        .collect::<collections::BTreeSet<_>>();

    let changes = service_names
        .into_iter()
        .flat_map(|service_name| {
            let removals = actual_containers
                .iter()
                .filter(|container| &container.service_name == service_name)
                .map(service_container_removal);

            match desired_services.get(service_name) {
                None => removals.collect(),

                Some(service_definition) => {
                    let additions = iter::repeat_with(|| {
                        service_container_addition(service_name, service_definition)
                    })
                    .take(service_definition.replica_count.into());

                    match service_definition.update_order {
                        model::OperationOrder::StartFirst => alternate(additions, removals),
                        model::OperationOrder::StopFirst => alternate(removals, additions),
                    }
                }
            }
        })
        .collect();

    simplify(changes)
}

fn service_container_removal(
    model::ActualContainer {
        container_id,
        service_config_hash,
        service_name,
    }: &model::ActualContainer,
) -> model::ServiceContainerChange {
    model::ServiceContainerChange::Remove {
        container_id: container_id.clone(),
        service_config_hash: service_config_hash.clone(),
        service_name: service_name.clone(),
    }
}

fn service_container_addition(
    service_name: &str,
    service_definition: &model::DesiredServiceDefinition,
) -> model::ServiceContainerChange {
    model::ServiceContainerChange::Add {
        service_config_hash: service_definition.service_config_hash.clone(),
        service_name: service_name.into(),
    }
}

fn alternate<T: IntoIterator<Item = V>, U: IntoIterator<Item = V>, V>(evens: T, odds: U) -> Vec<V> {
    let mut evens = evens.into_iter();
    let mut odds = odds.into_iter();

    let mut queue = vec![];

    loop {
        match (evens.next(), odds.next()) {
            (None, None) => break,
            (None, Some(odd)) => queue.push(odd),
            (Some(even), None) => queue.push(even),
            (Some(even), Some(odd)) => {
                queue.push(even);
                queue.push(odd);
            }
        }
    }

    queue
}

fn simplify(changes: Vec<model::ServiceContainerChange>) -> Vec<model::ServiceContainerChange> {
    let mut changes = collections::VecDeque::from(changes);
    let mut simplified_changes = vec![];

    loop {
        let simplified_change = match (changes.pop_front(), changes.pop_front()) {
            (None, None) => break,
            (None, Some(b)) => b,
            (Some(a), None) => a,

            (
                Some(model::ServiceContainerChange::Add {
                    service_config_hash: a_hash,
                    service_name: a_name,
                }),
                Some(model::ServiceContainerChange::Remove {
                    container_id,
                    service_config_hash: b_hash,
                    service_name: b_name,
                }),
            ) if a_hash == b_hash && a_name == b_name => model::ServiceContainerChange::Keep {
                container_id,
                service_config_hash: b_hash,
                service_name: b_name,
            },

            (
                Some(model::ServiceContainerChange::Remove {
                    container_id,
                    service_config_hash: a_hash,
                    service_name: a_name,
                }),
                Some(model::ServiceContainerChange::Add {
                    service_config_hash: b_hash,
                    service_name: b_name,
                }),
            ) if a_hash == b_hash && a_name == b_name => model::ServiceContainerChange::Keep {
                container_id,
                service_config_hash: a_hash,
                service_name: a_name,
            },

            (Some(a), Some(b)) => {
                changes.push_front(b);
                a
            }
        };

        simplified_changes.push(simplified_change);
    }

    simplified_changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case::test_case(
        "",
        "",
        "";
        "0 to 0"
    )]
    #[test_case::test_case(
        "",
        "Xa1±",
        "+Xa";
        "0 to 1, start first"
    )]
    #[test_case::test_case(
        "",
        "Xa1∓",
        "+Xa";
        "0 to 1, stop first"
    )]
    #[test_case::test_case(
        "Xa₀",
        "",
        "-Xa₀";
        "1 to 0"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁",
        "Xa2±",
        "=Xa₀ =Xa₁";
        "2 to 2, equal hash, start first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁",
        "Xa2∓",
        "=Xa₀ =Xa₁";
        "2 to 2, equal hash, stop first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁",
        "Xb2±",
        "+Xb -Xa₀ +Xb -Xa₁";
        "2 to 2, unequal hash, start first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁",
        "Xb2∓",
        "-Xa₀ +Xb -Xa₁ +Xb";
        "2 to 2, unequal hash, stop first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁ Xa₂",
        "Xa5±",
        "=Xa₀ =Xa₁ =Xa₂ +Xa +Xa";
        "3 to 5, equal hash, start first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁ Xa₂",
        "Xa5∓",
        "=Xa₀ =Xa₁ =Xa₂ +Xa +Xa";
        "3 to 5, equal hash, stop first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁ Xa₂",
        "Xb5±",
        "+Xb -Xa₀ +Xb -Xa₁ +Xb -Xa₂ +Xb +Xb";
        "3 to 5, unequal hash, start first"
    )]
    #[test_case::test_case(
        "Xa₀ Xa₁ Xa₂",
        "Xb5∓",
        "-Xa₀ +Xb -Xa₁ +Xb -Xa₂ +Xb +Xb +Xb";
        "3 to 5, unequal hash, stop first"
    )]
    #[test_case::test_case(
        "Xa₀ Yb₁ Yb₂ Zc₃ Zc₄",
        "Xd1∓ Yb3± Ze1∓",
        "-Xa₀ +Xd =Yb₁ =Yb₂ +Yb -Zc₃ +Ze -Zc₄";
        "multiple services"
    )]
    fn handles(
        actual_containers: &str,
        desired_services: &str,
        expected_changes: &str,
    ) -> anyhow::Result<()> {
        let actual_containers = actual_containers
            .split_whitespace()
            .map(|container| model::ActualContainer {
                container_id: (&container[2..]).into(),
                service_config_hash: (&container[1..2]).into(),
                service_name: (&container[..1]).into(),
            })
            .collect();
        let desired_services = desired_services
            .split_whitespace()
            .map(|service| {
                Ok((
                    (&service[..1]).into(),
                    model::DesiredServiceDefinition {
                        replica_count: service[2..3].parse()?,
                        service_config_hash: (&service[1..2]).into(),
                        update_order: match &service[3..] {
                            "±" => model::OperationOrder::StartFirst,
                            "∓" => model::OperationOrder::StopFirst,
                            update_order => anyhow::bail!("{update_order}"),
                        },
                    },
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        let expected_changes = expected_changes
            .split_whitespace()
            .map(|change| match &change[..1] {
                "+" => Ok(model::ServiceContainerChange::Add {
                    service_config_hash: (&change[2..]).into(),
                    service_name: (&change[1..2]).into(),
                }),
                "=" => Ok(model::ServiceContainerChange::Keep {
                    container_id: (&change[3..]).into(),
                    service_config_hash: (&change[2..3]).into(),
                    service_name: (&change[1..2]).into(),
                }),
                "-" => Ok(model::ServiceContainerChange::Remove {
                    container_id: (&change[3..]).into(),
                    service_config_hash: (&change[2..3]).into(),
                    service_name: (&change[1..2]).into(),
                }),
                operator => anyhow::bail!("{operator}"),
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        assert_eq!(go(&actual_containers, &desired_services), expected_changes);

        Ok(())
    }
}

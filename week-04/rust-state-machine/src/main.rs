mod serialization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Pending,
    Approved,
    Executed,
    Cancelled,
}

#[derive(Debug, PartialEq, Eq)]
enum VaultError {
    Unauthorized,
    InvalidTransition,
    ZeroAmount,
    RequestNotFound,
}

trait Authorizable {
    fn approve(&mut self, caller: &str) -> Result<(), VaultError>;
}

#[derive(Debug)]
struct Withdrawal<T> {
    owner: String,
    recipient: T,
    amount: u64,
    status: Status,
}

impl<T> Withdrawal<T> {
    fn new(owner: String, recipient: T, amount: u64) -> Result<Self, VaultError> {
        if amount == 0 {
            return Err(VaultError::ZeroAmount);
        }

        Ok(Self {
            owner,
            recipient,
            amount,
            status: Status::Pending,
        })
    }

    fn execute(&mut self) -> Result<(), VaultError> {
        if self.status != Status::Approved {
            return Err(VaultError::InvalidTransition);
        }

        self.status = Status::Executed;
        Ok(())
    }

    fn cancel(&mut self, caller: &str) -> Result<(), VaultError> {
        if caller != self.owner {
            return Err(VaultError::Unauthorized);
        }

        if self.status != Status::Pending {
            return Err(VaultError::InvalidTransition);
        }

        self.status = Status::Cancelled;
        Ok(())
    }
}

impl<T> Authorizable for Withdrawal<T> {
    fn approve(&mut self, caller: &str) -> Result<(), VaultError> {
        if caller != self.owner {
            return Err(VaultError::Unauthorized);
        }

        if self.status != Status::Pending {
            return Err(VaultError::InvalidTransition);
        }

        self.status = Status::Approved;
        Ok(())
    }
}

#[derive(Debug)]
struct WithdrawalRegistry<T> {
    requests: Vec<Withdrawal<T>>,
}

impl<T> WithdrawalRegistry<T> {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    fn create(&mut self, owner: String, recipient: T, amount: u64) -> Result<usize, VaultError> {
        let request = Withdrawal::new(owner, recipient, amount)?;
        let id = self.requests.len();
        self.requests.push(request);
        Ok(id)
    }

    fn request(&self, id: usize) -> Result<&Withdrawal<T>, VaultError> {
        self.requests.get(id).ok_or(VaultError::RequestNotFound)
    }

    fn approve(&mut self, id: usize, caller: &str) -> Result<(), VaultError> {
        self.requests
            .get_mut(id)
            .ok_or(VaultError::RequestNotFound)?
            .approve(caller)
    }

    fn execute(&mut self, id: usize, caller: &str) -> Result<(), VaultError> {
        let request = self
            .requests
            .get_mut(id)
            .ok_or(VaultError::RequestNotFound)?;
        if caller != request.owner {
            return Err(VaultError::Unauthorized);
        }
        request.execute()
    }

    fn cancel(&mut self, id: usize, caller: &str) -> Result<(), VaultError> {
        self.requests
            .get_mut(id)
            .ok_or(VaultError::RequestNotFound)?
            .cancel(caller)
    }
}

fn main() {
    let mut registry = WithdrawalRegistry::new();
    let id = registry
        .create("Alice".to_string(), "Bob".to_string(), 100)
        .unwrap();

    registry.approve(id, "Alice").unwrap();
    registry.execute(id, "Alice").unwrap();

    println!("{:?}", registry.request(id).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transition() {
        let mut w = Withdrawal::new("Alice".into(), "Bob", 100).unwrap();

        w.approve("Alice").unwrap();
        w.execute().unwrap();

        assert_eq!(w.status, Status::Executed);
    }

    #[test]
    fn cannot_execute_pending() {
        let mut w = Withdrawal::new("Alice".into(), "Bob", 100).unwrap();

        assert_eq!(w.execute(), Err(VaultError::InvalidTransition));
    }

    #[test]
    fn rejects_unauthorized_approval() {
        let mut w = Withdrawal::new("Alice".into(), "Bob", 100).unwrap();

        assert_eq!(w.approve("Mallory"), Err(VaultError::Unauthorized));
        assert_eq!(w.status, Status::Pending);
    }

    #[test]
    fn cancellation_is_terminal() {
        let mut w = Withdrawal::new("Alice".into(), "Bob", 100).unwrap();

        w.cancel("Alice").unwrap();

        assert_eq!(w.approve("Alice"), Err(VaultError::InvalidTransition));
    }

    #[test]
    fn executed_request_cannot_execute_again() {
        let mut w = Withdrawal::new("Alice".into(), "Bob", 100).unwrap();

        w.approve("Alice").unwrap();
        w.execute().unwrap();

        assert_eq!(w.execute(), Err(VaultError::InvalidTransition));
        assert_eq!(w.status, Status::Executed);
    }

    #[test]
    fn rejects_zero_amount_and_nonexistent_requests() {
        let mut registry = WithdrawalRegistry::<&str>::new();

        assert_eq!(
            registry.create("Alice".into(), "Bob", 0),
            Err(VaultError::ZeroAmount)
        );
        assert!(registry.requests.is_empty());
        assert!(matches!(
            registry.request(0),
            Err(VaultError::RequestNotFound)
        ));
        assert_eq!(
            registry.approve(0, "Alice"),
            Err(VaultError::RequestNotFound)
        );
        assert_eq!(
            registry.execute(0, "Alice"),
            Err(VaultError::RequestNotFound)
        );
        assert_eq!(
            registry.cancel(0, "Alice"),
            Err(VaultError::RequestNotFound)
        );
    }

    #[test]
    fn invalid_registry_operations_leave_state_unchanged() {
        let mut registry = WithdrawalRegistry::new();
        let id = registry.create("Alice".into(), "Bob", 100).unwrap();
        let original = registry.request(id).unwrap().status;

        assert_eq!(
            registry.approve(id, "Mallory"),
            Err(VaultError::Unauthorized)
        );
        assert_eq!(registry.request(id).unwrap().status, original);

        assert_eq!(
            registry.execute(id, "Alice"),
            Err(VaultError::InvalidTransition)
        );
        assert_eq!(registry.request(id).unwrap().status, original);

        assert_eq!(
            registry.cancel(id, "Mallory"),
            Err(VaultError::Unauthorized)
        );
        assert_eq!(registry.request(id).unwrap().status, original);
    }

    #[test]
    fn cancelled_request_cannot_execute() {
        let mut registry = WithdrawalRegistry::new();
        let id = registry.create("Alice".into(), "Bob", 100).unwrap();
        registry.cancel(id, "Alice").unwrap();

        assert_eq!(
            registry.execute(id, "Alice"),
            Err(VaultError::InvalidTransition)
        );
        assert_eq!(registry.request(id).unwrap().status, Status::Cancelled);
    }

    #[test]
    fn operation_sequences_preserve_state_invariants() {
        const ACTIONS: [&str; 6] = [
            "approve",
            "execute",
            "cancel",
            "unauthorized_approve",
            "unauthorized_cancel",
            "unauthorized_execute",
        ];
        const SEQUENCE_COUNT: usize = 6usize.pow(5);

        for mut sequence in 0..SEQUENCE_COUNT {
            let mut registry = WithdrawalRegistry::new();
            let id = registry.create("Alice".into(), "Bob", 100).unwrap();

            for _ in 0..5 {
                let action_index = sequence % ACTIONS.len();
                sequence /= ACTIONS.len();
                let action = ACTIONS[action_index];
                let before = registry.request(id).unwrap().status;

                let result = match action {
                    "approve" => registry.approve(id, "Alice"),
                    "execute" => registry.execute(id, "Alice"),
                    "cancel" => registry.cancel(id, "Alice"),
                    "unauthorized_approve" => registry.approve(id, "Mallory"),
                    "unauthorized_cancel" => registry.cancel(id, "Mallory"),
                    "unauthorized_execute" => registry.execute(id, "Mallory"),
                    _ => unreachable!(),
                };

                let after = registry.request(id).unwrap().status;
                if result.is_err() {
                    assert_eq!(after, before, "{action} changed state on failure");
                }

                match after {
                    Status::Pending | Status::Approved | Status::Executed | Status::Cancelled => {}
                }
                if before == Status::Executed {
                    assert_eq!(after, Status::Executed);
                }
                if before == Status::Cancelled {
                    assert_eq!(after, Status::Cancelled);
                }
                if after == Status::Approved {
                    assert!(before == Status::Pending || before == Status::Approved);
                }
                if after == Status::Executed {
                    assert!(before == Status::Approved || before == Status::Executed);
                }
                if after == Status::Cancelled {
                    assert!(before == Status::Pending || before == Status::Cancelled);
                }
            }
        }
    }
}

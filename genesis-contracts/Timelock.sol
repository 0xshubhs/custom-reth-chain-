// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Timelock - Delay-enforcing governance contract for Meowchain
/// @notice Queues operations with a mandatory delay before execution.
///         Governance Safe proposes; after minDelay passes, anyone can execute.
contract Timelock {
    /// @notice Minimum delay (in seconds) between queueing and executing
    uint256 public minDelay;

    /// @notice Address that can queue operations (Governance Safe)
    address public proposer;

    /// @notice Address that can execute operations (Governance Safe)
    address public executor;

    /// @notice Admin that can change delay and roles
    address public admin;

    /// @notice Whether the contract is paused (emergency only)
    bool public paused;

    /// @notice Mapping from operation hash to timestamp when it becomes executable (0 = not queued)
    mapping(bytes32 => uint256) public timestamps;

    /// @notice A queued operation is ready at this timestamp
    uint256 private constant _DONE_TIMESTAMP = uint256(1);

    event OperationScheduled(bytes32 indexed id, address target, uint256 value, bytes data, uint256 delay);
    event OperationExecuted(bytes32 indexed id, address target, uint256 value, bytes data);
    event OperationCancelled(bytes32 indexed id);
    event MinDelayChanged(uint256 oldDelay, uint256 newDelay);
    event Paused(address account);
    event Unpaused(address account);

    modifier onlyProposer() {
        require(msg.sender == proposer, "Timelock: caller is not proposer");
        _;
    }

    modifier onlyExecutor() {
        require(msg.sender == executor, "Timelock: caller is not executor");
        _;
    }

    modifier onlyAdmin() {
        require(msg.sender == admin, "Timelock: caller is not admin");
        _;
    }

    modifier whenNotPaused() {
        require(!paused, "Timelock: paused");
        _;
    }

    constructor(uint256 _minDelay, address _proposer, address _executor, address _admin) {
        minDelay = _minDelay;
        proposer = _proposer;
        executor = _executor;
        admin = _admin;
    }

    /// @notice Hash an operation for identification
    function hashOperation(address target, uint256 value, bytes calldata data, bytes32 salt)
        public pure returns (bytes32)
    {
        return keccak256(abi.encode(target, value, data, salt));
    }

    /// @notice Queue an operation with the minimum delay
    function schedule(address target, uint256 value, bytes calldata data, bytes32 salt, uint256 delay)
        external onlyProposer whenNotPaused
    {
        require(delay >= minDelay, "Timelock: insufficient delay");
        bytes32 id = hashOperation(target, value, data, salt);
        require(timestamps[id] == 0, "Timelock: operation already scheduled");
        timestamps[id] = block.timestamp + delay;
        emit OperationScheduled(id, target, value, data, delay);
    }

    /// @notice Execute a queued operation after its delay has passed
    function execute(address target, uint256 value, bytes calldata data, bytes32 salt)
        external payable onlyExecutor whenNotPaused
    {
        bytes32 id = hashOperation(target, value, data, salt);
        require(timestamps[id] > _DONE_TIMESTAMP, "Timelock: operation not ready");
        require(block.timestamp >= timestamps[id], "Timelock: operation not yet ready");
        timestamps[id] = _DONE_TIMESTAMP;
        (bool success,) = target.call{value: value}(data);
        require(success, "Timelock: underlying call reverted");
        emit OperationExecuted(id, target, value, data);
    }

    /// @notice Cancel a queued operation
    function cancel(bytes32 id) external onlyProposer {
        require(timestamps[id] > _DONE_TIMESTAMP, "Timelock: operation cannot be cancelled");
        timestamps[id] = 0;
        emit OperationCancelled(id);
    }

    /// @notice Update the minimum delay (must go through timelock itself)
    function updateDelay(uint256 newDelay) external onlyAdmin {
        emit MinDelayChanged(minDelay, newDelay);
        minDelay = newDelay;
    }

    /// @notice Pause the timelock (emergency only, admin can bypass)
    function pause() external onlyAdmin {
        paused = true;
        emit Paused(msg.sender);
    }

    /// @notice Unpause the timelock
    function unpause() external onlyAdmin {
        paused = false;
        emit Unpaused(msg.sender);
    }

    /// @notice Check if an operation is pending (queued but not executed)
    function isOperationPending(bytes32 id) public view returns (bool) {
        return timestamps[id] > _DONE_TIMESTAMP;
    }

    /// @notice Check if an operation is ready to execute
    function isOperationReady(bytes32 id) public view returns (bool) {
        return timestamps[id] > _DONE_TIMESTAMP && block.timestamp >= timestamps[id];
    }

    /// @notice Check if an operation has been executed
    function isOperationDone(bytes32 id) public view returns (bool) {
        return timestamps[id] == _DONE_TIMESTAMP;
    }

    receive() external payable {}
}

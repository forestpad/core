import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CoreProject } from "../target/types/core_project";
import { expect } from "chai";

describe("core-project", () => {
  // 프로바이더와 연결 설정
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 프로그램 ID를 사용하여 프로그램 객체 가져오기
  const program = anchor.workspace.CoreProject as Program<CoreProject>;

  // 테스트에 사용할 지갑
  const wallet = provider.wallet as anchor.Wallet;

  it("Initialize 테스트", async () => {
    // 초기화 트랜잭션 실행
    const tx = await program.methods
        .initialize()
        .accounts({})
        .rpc();

    console.log("초기화 트랜잭션:", tx);
    // 트랜잭션이 성공적으로 완료되었으면 테스트 통과
  });

  it("지갑 정보 확인 테스트", async () => {
    // 지갑 정보 확인 트랜잭션 실행
    const tx = await program.methods
        .checkWalletInfo()
        .accounts({
          wallet: wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

    console.log("지갑 정보 확인 트랜잭션:", tx);
    console.log("지갑 주소:", wallet.publicKey.toString());

    // 트랜잭션이 성공적으로 완료되었는지 확인
    const confirmedTx = await provider.connection.getParsedTransaction(tx, "confirmed");
    expect(confirmedTx).to.not.be.null;
  });

  it("메시지 저장 및 조회 테스트", async () => {
    // 메시지 계정 PDA 찾기
    const [messageAccount] = await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("message"), wallet.publicKey.toBuffer()],
        program.programId
    );

    // 테스트용 메시지
    const testMessage = "안녕하세요, 솔라나 블록체인!";

    // 메시지 저장 트랜잭션 실행
    const tx = await program.methods
        .saveMessage(testMessage)
        .accounts({
          author: wallet.publicKey,
          messageAccount: messageAccount,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

    console.log("메시지 저장 트랜잭션:", tx);

    // 저장된 메시지 조회
    const messageData = await program.account.messageAccount.fetch(messageAccount);

    // 저장된 메시지 검증
    expect(messageData.author.toString()).to.equal(wallet.publicKey.toString());
    expect(messageData.message).to.equal(testMessage);

    // BN 타입 체크 - timestamp는 BN 타입입니다
    expect(messageData.timestamp).to.not.be.null;
    expect(messageData.timestamp.toNumber).to.be.a('function');

    const timestamp = messageData.timestamp.toNumber();
    expect(timestamp).to.be.a('number');
    expect(timestamp).to.be.greaterThan(0);

    console.log("저장된 메시지:", messageData.message);
    console.log("작성자:", messageData.author.toString());
    console.log("작성 시간:", new Date(timestamp * 1000).toLocaleString());
  });
});
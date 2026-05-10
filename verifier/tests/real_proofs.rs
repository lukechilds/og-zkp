use assert_cmd::prelude::*;
use serde_json::Value;
use std::process::Command;

const BLOCK_INCLUSION_ROOT: &str =
    "645193f7e45302f503f14d6bdc593a12ee954b5ca844d38affaae51febb77a3e";
const IDENTITY: &str = "x.com/lukechilds";

const P2PKH_PROOF: &str = "og-zkp1r79ssqqqqqqqqq8l6hg9kjznqyvq0u9nrkn5kynn68yvt0z5c7rt3dfkke6gjgsdwk2qcnd3sy28t955mvdrrgstatx95zmpzkxssk44segjkdr6rpyc5kttngkyx6zskjkc0sfvxvazm5n92la3k00czcl04p7lcvr37lqhx6qmpz9ley9e59wdule7n4h8kd4m8k4m2vd3u7v5wu4mudp9kln8a88e4maffj0fp68el5acs2wr0yev2wj5dvxml2zgurr29ak3lwtu2dehyf2tn6vfe2qu325j49yk020m3ves454ryftsyua6cgd0kkp7ln77lxf284kheg0r90d0mq7cnv4yaz7up4rtd36rj8tzn4uyhng6k2j7haxqp8khm5dmv008hftd04un7pena4apkym50tx0fhpp9elk7vkjxqdyuxvtcu9qfr5r3nd47f6hle5pjx6s4let3zw88x3mxr67j3tqwlx3u4mr9fvwwu6uxa42e4tauuatxzpnknte3g59rlxz97lyhv468yljhl262jmmnx8hke74mave8hpaldx05cjv0zg9h93z04ahj6jf98dmcej5trnfy8t3vvcg0a0u7xv2pz5c9vl98wyytp44nzlphlc28h2gyya7ggwwvfanvyry37qvx4sgq28ggqzncc8qyfkgxfhssgdy95q9wdcqwf3qzulyq87v073qe5npjdp9utzhezj9y22p43q864zzndx3hrv8pha73lg833avhsh99kqudlh6d3j7ya268rceg554ms0qum8s207509k940v2zyfxh22wa7kl94lspe2tacmnjqcqqq43h058";
const P2SH_P2WPKH_PROOF: &str = "og-zkp1r79ssqqqqqqqqq8l6h847jznqy2qdu8tsckeh9dv6p2dvn9kqcex6m3dhw9kag9ryku5syj8kyef3jny66fgk22r26hycx7wvf9kurark2knl7kjzgsny2v4dw9jpfqks8x635gjsejy4nvp5h6j20hcycl04s00u8qgzue7p6qjgyyte856674nf5l77966htugj9pnwt7k9wtlae47jmw0ru40e6tmsz0mmkdtkfxxcnfua8tzm3ql284f3jfmev8pn7n7krahjmfug5dmltha0ehfa4fkum059dukva7taelcmxf822zx0ghdlcsffparv4e9vlwdrqlze6503t9avqedh23l0lqnmgf5gax3j8ava78ddfd98h0twv2cmlsdfnn4x0h62s7ws232hpyvhy6r38mf8nmkqvy7vnf9d8mc4e6rt3nwpp0r26vjn0k50zkz5289qjwda7ncrxldx6v58xu6t65djdq6ehcua6r63gxjttc8p25kzkwl3e43tn6jazgtf7gtu0x99zr647uz4znd6kvh0wcx2t0l4s47usy2zexttjncje03k3ev2n0y82h0m49ggtn0cf2umufrvzchaek87ygjffm8v8pcg6kryzdtm8ej6rv42q9xazzrqnl9lejqpjurqqe6xrpns8q8x5yxqz36uycp3cppdqsqsrhcpv7lqr2yxqy5lsq2z3kgqly49f05nmka9kt3x3q2umrregl9ejhd57nnjhg4g56mfyhhl8nx7h7kt6tm5mrhs8a8qvfrtkfacf9gwf0ap4y3ztf8lpex5lpd74d2njelpm0cvkpeqvqqq6el2lt";
const P2WPKH_PROOF: &str = "og-zkp1r79ssqqqqqqqqq8l6h856jynqyvq0urh0v5m20xgnj27t26fd6feexmxf6m58qvg5hphp6kjd2zw92k4rshzeunrtqvmyxxd4jv3g3vf2etp9pp3s2x4yd3n8fydv2n5echafzmnc0uu67mslc2slhky5dlls8g70rs8nljvqfs3p6ee8rwl3x2h0rzaz9n90t9hmh3wvn7t6sltm9t0w02dmy04yfse3syg6dsyn05mxkhm3znky2rr3fl0jjt8xndwcrt6tvcrkhx4ketmpmrmzkh73jafj9u2t52awhjz87gpwwy2najlczd749gmldf3npejtx95ntdaqwec0f086ddrhu5t4meku06vuy9t8wx5dzxmdzk0le536nhnpe4670drvk42paxthwwdx3gdwy53jkvtchld3shcyw02lg3s5mzltv0eh6qhwkh5tl4uamge4xhwe8wvdk4pem6hwnc3la9hcjrv4umuwvwj684pnghwqua7d8yvcz65w6n6w04seyht3775mh0p335jgadv72zdht0clfezpnznz3lmlj4t3ugu5au3lvff9l6xfrzwhjd6e00cavvd7wz0qhtgm2dfjhnv3njp3tp660uegakr8r8daj7auxy5yqp3xtxpqjk96ekzq2gwpqmv2zpshn5pq3vwppf8yqzst0yrw3cgqq0u7qqjvgg2mncqt2hqqnkgr38yunzzgu027xzqfqsgc9wzz2pf6vyykma297263t5k9t9nw06lmewkpjg0zlxscppcval8qf8z7l0mk7fk3mj4d5xy9xn283ltmfymmdh2kg9qazrgcyusxqqqxlkjwx";
const P2TR_PROOF: &str = "og-zkp1r79ssqqqqqqqqq8l6h846jznqyvqdc8rc6vmnze6g3nvadjed5vnzn7mjzt9yrtjvy554vk5vn34capxdhxrlyyct834qwqxn9wmgtqsjdfjej23fnazzwg3c2cfyjr9a5324sg2e6593xy98l469tdharpthuxjz70gkre7lzl9ws3q88zd79pjw33l3lj3lawknt4aaz4ze78xwkl9e7x5gf8yeacgj9q42dm45mq0sdfn06ypazre6skx7kpzd47l9vfr0khvayehsjee7jfd0augfd8v6rxncur3mlfy8us0k2xzdk4sakzyge7a8kkpjtxe6qctahmm45cetnug5ujyrkn8em0zkgetk0xs0phdvxhd05v7xltprfr2de057n3nxdkuh0duv248ylw3ekurx9xf83fve4shd0pth2s4p9jn2hfvymzt3k42fekl5yjaxn20jtzdhfyz53z3a8vy8k6tal8j5nlfctflayfgm0y3288mc8v2d09jtlkkz4t67vf2kuy6hrg45a7al8j9dujk6e7zncxlzts97fdwjslprw8dcj9qe7d9nfe72vhfq4ndd28ttag2aumaws4n80r2u2749wlnktv54tpj8x4ajzl7rvw9qrcptqrprljujmqzyj8p802gyxlz6qxptvq3uesqq6kqpemuqq33hqy9jzzpzt2fpqk86qpswsgwqefenh45eajdf6cfstpdcndnr4khkkj0hdd43fat6hgmtuhe4myjdctcu800ndaealn976ptefre7nvkgj6gyf604jfvf8k0cht49tllwhusmdn8rearjqcqqqzerrw7";

struct Fixture {
    name: &'static str,
    proof: &'static str,
    block_month: &'static str,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "p2pkh",
        proof: P2PKH_PROOF,
        block_month: "1538352000",
    },
    Fixture {
        name: "p2sh-p2wpkh",
        proof: P2SH_P2WPKH_PROOF,
        block_month: "1551398400",
    },
    Fixture {
        name: "p2wpkh",
        proof: P2WPKH_PROOF,
        block_month: "1551398400",
    },
    Fixture {
        name: "p2tr",
        proof: P2TR_PROOF,
        block_month: "1740787200",
    },
];

#[test]
fn verifies_real_proof_fixtures() {
    for fixture in FIXTURES {
        let output = Command::cargo_bin("og-zkp-verifier")
            .expect("verifier binary exists")
            .arg(fixture.proof.trim())
            .output()
            .expect("verifier runs");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let name = fixture.name;
        assert!(
            output.status.success(),
            "fixture {name} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );

        let json: Value = serde_json::from_slice(&output.stdout).expect("stdout is valid JSON");
        assert_eq!(json["valid"].as_bool(), Some(true), "{name}");
        assert_eq!(json["block_inclusion_root"], BLOCK_INCLUSION_ROOT, "{name}");
        assert_eq!(json["block_month"], fixture.block_month, "{name}");
        assert_eq!(json["identity"], IDENTITY, "{name}");
        assert!(json.get("error").is_none(), "{name}");
    }
}

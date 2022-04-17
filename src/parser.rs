use regex::{Regex};

#[derive(PartialEq,Debug)]
enum Pattern{
    //1..とか
    NPP,
    //..1とか
    PPN,
    //1..3とか
    NPPN,
    //1,3..とか
    NCNPP,
    //..1,3とか
    PPNCN,
    //1,3..7とか
    NCNPPN,
    //1..5,7とか
    NPPNCN,
}

//正規表現からキャプチャした数値をi32のベクタにして返す
fn cap2vec(caps:&regex::Captures)->Result<Vec<i32>,&'static str>{
    if caps.len()<1{
        return Err("置換の入力が不正");
    }
    let mut res=vec![];
    for i in 1..caps.len(){
        let p=caps[i].parse::<i32>();
        if let Err(_)=p{
            return Err("置換の入力が不正");
        }
        res.push(p.unwrap());
    }
    Ok(res)
}



//数列の置換をパースする
/*
    "1.."   -> 1,2,3..
    "1,3.." -> 1,3,5..
    "1,-1.."-> "1,-1,-3.."
    "..5"   -> ..3,4,5
    "..7,5" -> ..9,7,5
    こんなかんじ
*/
// ({置換文字列},fn(idx)->val)のベクタを返す
pub fn parse_sequence(v:Vec<(String,String)>,num:u32)->Result<Vec<(String,Box<dyn Fn(i32)->i32>)>,&'static str>{
    let mut res:Vec<(String,Box<dyn Fn(i32)->i32>)>=vec![];
    for i in v{
        //どちらか空欄なら飛ばす
        if i.0.len()==0 || i.1.len()==0{
            continue;
        }
        //置換先
        let mut rs=i.1;
        //空白は無視する
        rs.retain(|c| {c != ' ' && c !='\t'});

        //入力パターン
        let ptn;

        //形式のチェックとともに数字を切り出す
        let caps=
            if let Some(cap)=Regex::new(r"^(-?\d+)\.\.$").unwrap().captures(&rs){
                ptn=Pattern::NPP;
                cap
            }else if let Some(cap)=Regex::new(r"^\.\.(-?\d+)$").unwrap().captures(&rs){
                ptn=Pattern::PPN;
                cap
            }else if let Some(cap)=Regex::new(r"^(-?\d+)\.\.(-?\d+)$").unwrap().captures(&rs){
                ptn=Pattern::NPPN;
                cap
            }else if let Some(cap)=Regex::new(r"^(-?\d+),(-?\d+)\.\.$").unwrap().captures(&rs){
                ptn=Pattern::NCNPP;
                cap
            }else if let Some(cap)=Regex::new(r"^\.\.(-?\d+),(-?\d+)$").unwrap().captures(&rs){
                ptn=Pattern::PPNCN;
                cap
            }else if let Some(cap)=Regex::new(r"^(-?\d+)\.\.(-?\d+),(-?\d+)$").unwrap().captures(&rs){
                ptn=Pattern::NPPNCN;
                cap
            }else if let Some(cap)=Regex::new(r"^(-?\d+),(-?\d+)\.\.(-?\d+)$").unwrap().captures(&rs){
                ptn=Pattern::NCNPPN;
                cap
            }else{
                return Err("置換の入力が不正");
            };
        
        let caps=cap2vec(&caps);
        if let Err(e)=caps{
            return Err(e);
        }
        let caps=caps.unwrap();

        match ptn{
            //1..とか
            Pattern::NPP=>{
                let lmd=move |idx:i32|{caps[0]+idx};
                res.push((i.0,Box::new(lmd)));
            },
            //1,3..とか
            Pattern::NCNPP=>{
                let diff=caps[1]-caps[0];
                let lmd=move |idx:i32|{caps[0]+idx*diff};
                res.push((i.0,Box::new(lmd)));
            },
            //..3とか
            Pattern::PPN=>{
                let lmd=move |idx:i32|{caps[0]-(num as i32-1)+idx};
                res.push((i.0,Box::new(lmd)));
            },
            //..3,5とか
            Pattern::PPNCN=>{
                let diff=caps[1]-caps[0];
                let lmd=move |idx:i32|{caps[1]-(num as i32-1-idx)*diff};
                res.push((i.0,Box::new(lmd)));
            },
            //1..3とか
            Pattern::NPPN=>{
                if caps[0]<caps[1]{
                    let lmd=move |idx:i32|{caps[0]+idx%(caps[1]-caps[0]+1)};
                    res.push((i.0,Box::new(lmd)));
                }else{
                    let lmd=move |idx:i32|{caps[0]-(idx%(caps[0]-caps[1]+1))};
                    res.push((i.0,Box::new(lmd)));
                }
            },
            //1..3,5とか
            Pattern::NPPNCN=>{
                if (caps[0]<caps[1] && caps[1]<caps[2]) || (caps[0]>caps[1] && caps[1]>caps[2]){
                    let diff=caps[2]-caps[1];
                    let cyc=(caps[2]-caps[0])/diff;
                    let lmd=move |idx:i32|{caps[0]+(caps[2]-caps[0])%diff+diff*(idx%(cyc+1))};
                    res.push((i.0,Box::new(lmd)));
                }else{
                    return Err("置換の入力が不正");
                }
            },
            //1,3..5とか
            Pattern::NCNPPN=>{
                if (caps[0]<caps[1] && caps[1]<caps[2]) || (caps[0]>caps[1] && caps[1]>caps[2]){
                    let diff=caps[1]-caps[0];
                    let cyc=(caps[2]-caps[0])/diff;
                    let lmd=move |idx:i32|{caps[0]+diff*(idx%(cyc+1))};
                    res.push((i.0,Box::new(lmd)));
                }else{
                    return Err("置換の入力が不正");
                }
            }
        }
    }
    Ok(res)
}